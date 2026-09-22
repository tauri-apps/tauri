// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Launches the built example and attaches Playwright to it over the Chrome
 * DevTools Protocol.
 *
 * The app is a Chromium, so it is driven the way a Chromium is driven: the
 * runtime starts Chromium's DevTools server on the port
 * `CEF_EXAMPLE_REMOTE_DEBUGGING_PORT` names (`Cef::remote_debugging`), and
 * Playwright's `chromium.connectOverCDP` attaches to it. Every CEF browser the
 * app creates — the main window, a Tauri window it opens, each child webview of
 * a multiwebview window, and a popup CEF opened itself — is a page target on that
 * connection, so the whole Playwright API works on each of them. No WebDriver,
 * no driver binary, and nothing added to the app for the tests' sake.
 */

import { spawn, spawnSync, type ChildProcess } from 'node:child_process'
import fs from 'node:fs'
import net from 'node:net'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium, type Browser, type Page } from '@playwright/test'

const dirname = path.dirname(fileURLToPath(import.meta.url))

/** `examples/cef`, the app under test. */
export const exampleDir = path.resolve(dirname, '..')
const repoRoot = path.resolve(exampleDir, '..', '..')
const targetDir = process.env.CARGO_TARGET_DIR ?? path.join(repoRoot, 'target')

/**
 * The origin the CEF runtime serves the app's own pages from. The system
 * webviews differ per platform (`tauri://localhost` on some, `http://tauri.localhost`
 * on others); CEF always uses `http://<scheme>.localhost`.
 */
export const APP_ORIGIN = 'http://tauri.localhost'

/**
 * What `pnpm tauri build --debug` produced: the `.app` bundle on macOS, where
 * CEF launches its helper apps by path from inside the bundle and so cannot run
 * as a bare executable, and the bare executable elsewhere, with the CEF
 * distribution laid out next to it by the `cef` crate's build script.
 */
export function builtApp(): string {
  if (process.env.CEF_E2E_APP_PATH) {
    return path.resolve(process.env.CEF_E2E_APP_PATH)
  }
  switch (process.platform) {
    case 'darwin':
      return path.join(
        targetDir,
        'debug',
        'bundle',
        'macos',
        'Tauri CEF Example.app'
      )
    case 'win32':
      return path.join(targetDir, 'debug', 'cef-example.exe')
    default:
      return path.join(targetDir, 'debug', 'cef-example')
  }
}

/** The executable inside {@link builtApp}. */
export function appExecutable(): string {
  const app = builtApp()
  return app.endsWith('.app')
    ? path.join(app, 'Contents', 'MacOS', 'cef-example')
    : app
}

export interface LaunchOptions {
  /**
   * Environment for the app, on top of what {@link CefApp.launch} sets: the
   * `CEF_EXAMPLE_*` knobs the example's README lists.
   */
  env?: Record<string, string>
}

/** One page target, as the DevTools server's `/json/list` reports it. */
interface Target {
  id: string
  type: string
  url: string
  webSocketDebuggerUrl: string
}

/** Whether `url` is the main window's page. */
function isMainPage(url: URL): boolean {
  return (
    url.origin === APP_ORIGIN
    && (url.pathname === '/' || url.pathname === '/index.html')
  )
}

/** One running instance of the example, and the protocol connections to it. */
export class CefApp {
  /** The main window's page. */
  main!: Page

  /**
   * Every Playwright connection to the app. The first one is made at launch;
   * {@link waitForPage} adds one whenever it needs a browser that appeared
   * later — see there for why.
   */
  private readonly connections: Browser[] = []
  /** The targets the DevTools server listed when the last connection was made. */
  private lastTargetsKey = ''

  private constructor(
    private readonly child: ChildProcess,
    /** Everything the app wrote to stdout and stderr. */
    readonly output: string[],
    /** The port Chromium's DevTools server listens on. */
    readonly port: number,
    /**
     * The profile directory of this run (`root_cache_path`): a fresh temporary
     * one, so nothing Chromium persists — the permission decisions the app's
     * handler makes above all — carries over into the next test.
     */
    readonly cacheDir: string,
    private readonly env: NodeJS.ProcessEnv,
    /**
     * The Playwright side of the current main connection: the one made at
     * launch, until {@link evaluateDetached} replaces it.
     */
    public browser: Browser
  ) {
    this.connections.push(browser)
  }

  static async launch(options: LaunchOptions = {}): Promise<CefApp> {
    const executable = appExecutable()
    const port = await freePort()
    const cacheDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tauri-cef-e2e-'))
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      CEF_EXAMPLE_REMOTE_DEBUGGING_PORT: String(port),
      CEF_EXAMPLE_CACHE_DIR: cacheDir,
      // A `tauri build --debug` binary is not a development build, so
      // `SecretStorage::Auto` would pick the OS secret store: on macOS the bundle
      // is ad-hoc signed and the keychain prompts on every rebuild, and CI
      // machines have no keyring at all. The mock keychain only weakens cookie
      // encryption at rest, which the test profile has no use for.
      CEF_EXAMPLE_SECRET_STORAGE: 'mock',
      ...options.env
    }

    const output: string[] = []
    const child = spawn(executable, [], {
      env,
      cwd: exampleDir,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    child.stdout?.on('data', (chunk) => output.push(String(chunk)))
    child.stderr?.on('data', (chunk) => output.push(String(chunk)))
    const exited = () => child.exitCode !== null || child.signalCode !== null

    let browser: Browser | undefined
    try {
      await waitForDevToolsServer(port, exited)
      browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
      const app = new CefApp(child, output, port, cacheDir, env, browser)
      app.main = await app.waitForPage(isMainPage)
      await app.main.waitForLoadState('load')
      // The page's script subscribes to the app's `cef://event` stream and then
      // fills the runtime badges; once those are in, everything a test triggers
      // is going to be reported.
      await app.main
        .locator('#chrome-version', { hasText: /Chromium \d/ })
        .waitFor()
      return app
    } catch (error) {
      await browser?.close().catch(() => {})
      await terminate(child)
      fs.rmSync(cacheDir, { recursive: true, force: true, maxRetries: 3 })
      const message = error instanceof Error ? error.message : String(error)
      throw new Error(
        `${message}\n--- output of ${executable} ---\n${output.join('')}`
      )
    }
  }

  /**
   * Every page Playwright has, across all connections. A browser that more than
   * one connection attached to is in here once per connection.
   */
  pages(): Page[] {
    return this.connections.flatMap((browser) =>
      browser.contexts().flatMap((context) => context.pages())
    )
  }

  /** The page targets the DevTools server lists right now. */
  private async targets(): Promise<Target[]> {
    const response = await fetch(`http://127.0.0.1:${this.port}/json/list`)
    return (await response.json()) as Target[]
  }

  /**
   * Waits for a page whose URL satisfies `matches` to exist. Every window the
   * app opens is one, and so are the child webviews of a multiwebview window
   * and a popup CEF opened on its own.
   *
   * A browser created after a connection was made needs a connection of its
   * own. CEF announces a new browser to the protocol as a target of type
   * `other` and re-types it `page` only once the runtime's placeholder document
   * is in, and Playwright ignores a target that was `other` when it attached.
   * A fresh connection attaches to every target with its current type, so once
   * the DevTools server lists a `page` with the wanted URL, one is made for it.
   */
  async waitForPage(
    matches: (url: URL) => boolean,
    { timeout = 20_000 }: { timeout?: number } = {}
  ): Promise<Page> {
    const matchesUrl = (url: string) => {
      try {
        return matches(new URL(url))
      } catch {
        return false
      }
    }
    const deadline = Date.now() + timeout
    for (;;) {
      const page = this.pages().find((page) => matchesUrl(page.url()))
      if (page) {
        return page
      }
      if (this.child.exitCode !== null || this.child.signalCode !== null) {
        throw new Error(
          `the app exited (code ${this.child.exitCode}, signal ${this.child.signalCode}) before the page appeared`
        )
      }
      if (Date.now() > deadline) {
        const open = this.pages().map((page) => page.url())
        throw new Error(
          `no page matched within ${timeout}ms; open pages: ${open.join(', ') || 'none'}`
        )
      }
      // One connection per set of targets: the server may list a page under
      // the URL it was asked for while Playwright reports where it ended up,
      // and reconnecting would not change that.
      const targets = await this.targets()
      const key = targets
        .map((target) => `${target.type} ${target.url}`)
        .sort()
        .join('\n')
      if (
        key !== this.lastTargetsKey
        && targets.some(
          (target) => target.type === 'page' && matchesUrl(target.url)
        )
      ) {
        await this.connect()
      }
      await sleep(100)
    }
  }

  /** Makes one more Playwright connection, which attaches to every target. */
  private async connect(): Promise<Browser> {
    this.lastTargetsKey = (await this.targets())
      .map((target) => `${target.type} ${target.url}`)
      .sort()
      .join('\n')
    const browser = await chromium.connectOverCDP(
      `http://127.0.0.1:${this.port}`
    )
    this.connections.push(browser)
    return browser
  }

  /**
   * Evaluates `expression` in the main page, with a user gesture, while no
   * Playwright connection to the app exists, then reconnects — once `until`,
   * evaluated there too, holds. `main` and `browser` are new objects afterwards.
   *
   * A popup CEF opens on its own (`NewWindowResponse::Allow`) never finishes
   * being created when a protocol client attaches to it the moment it appears:
   * the opener's `window.open` never returns, and the popup keeps no URL. A
   * client attaching during its first navigation fails that navigation instead.
   * Playwright attaches to every new target exactly that early, so for the
   * `window.open` that asks for such a popup the connections are dropped, the
   * expression runs over a bare session to the main page, and Playwright comes
   * back only when `until` says the popup has loaded — from then on it is a
   * page like any other.
   */
  async evaluateDetached(expression: string, until?: string): Promise<unknown> {
    await this.disconnect()
    const main = (await this.targets()).find((target) => {
      try {
        return isMainPage(new URL(target.url))
      } catch {
        return false
      }
    })
    if (!main) {
      throw new Error('the main page is not among the DevTools targets')
    }
    const session = await BareSession.open(main.webSocketDebuggerUrl)
    try {
      const value = await session.evaluate(expression)
      if (until) {
        const deadline = Date.now() + 15_000
        while (!(await session.evaluate(until))) {
          if (Date.now() > deadline) {
            throw new Error(`still false after 15s: ${until}`)
          }
          await sleep(100)
        }
      }
      return value
    } finally {
      session.close()
      this.browser = await this.connect()
      this.main = await this.waitForPage(isMainPage)
    }
  }

  /**
   * Closes the browser behind the target whose URL satisfies `matches`, through
   * the DevTools server: for a browser Playwright has no usable page for.
   */
  async closeTarget(matches: (url: URL) => boolean): Promise<void> {
    const target = (await this.targets()).find((target) => {
      try {
        return matches(new URL(target.url))
      } catch {
        return false
      }
    })
    if (!target) {
      throw new Error('no target matched')
    }
    const response = await fetch(
      `http://127.0.0.1:${this.port}/json/close/${target.id}`
    )
    if (!response.ok) {
      throw new Error(`could not close ${target.url}: ${response.status}`)
    }
  }

  /** Drops every Playwright connection, giving a wedged browser five seconds at most. */
  private async disconnect(): Promise<void> {
    const connections = this.connections.splice(0)
    await Promise.race([
      Promise.all(
        connections.map((browser) => browser.close().catch(() => {}))
      ),
      sleep(5_000)
    ])
  }

  /**
   * Delivers a deep link to this running instance, the way each platform does.
   */
  async openDeepLink(url: string): Promise<void> {
    if (process.platform === 'darwin') {
      // Launch Services hands the URL to the running application through
      // `application:openURLs:` — the path `open tauri-cef-example://hello` takes
      // once the bundle is registered for the scheme. Naming the bundle skips
      // the registration.
      const result = spawnSync('open', ['-a', builtApp(), url], {
        encoding: 'utf8'
      })
      if (result.status !== 0) {
        throw new Error(`\`open -a\` failed: ${result.stderr}`)
      }
      return
    }

    // Running the executable again with the URL as its argument, as a protocol
    // handler registration would: the second process finds Chromium's process
    // singleton of the first (they share a profile directory), relays its
    // command line to it and exits. That command line was cleared by
    // `command_line_args_disabled`; the runtime restores the URL onto it. The
    // debugging port is withheld so the newcomer never competes for it.
    const { CEF_EXAMPLE_REMOTE_DEBUGGING_PORT: _, ...env } = this.env
    const relaunch = spawn(appExecutable(), [url], {
      env,
      cwd: exampleDir,
      stdio: 'ignore'
    })
    const exited = new Promise<void>((resolve) =>
      relaunch.once('exit', () => resolve())
    )
    await Promise.race([
      exited,
      sleep(15_000).then(() => {
        relaunch.kill('SIGKILL')
      })
    ])
  }

  async close(): Promise<void> {
    // The browser was connected to, not launched, so this only disconnects.
    await this.disconnect()
    await terminate(this.child)
    fs.rmSync(this.cacheDir, { recursive: true, force: true, maxRetries: 5 })
  }
}

/** What the protocol answers a request with. */
interface Reply {
  id: number
  result?: { result: { value?: unknown }; exceptionDetails?: unknown }
  error?: { message: string }
}

/**
 * A WebSocket of our own to one target: a protocol client that attaches to
 * nothing else, for what has to happen while Playwright is not around.
 */
class BareSession {
  private nextId = 1
  private readonly pending = new Map<number, (reply: Reply) => void>()

  private constructor(private readonly socket: WebSocket) {
    socket.addEventListener('message', (message) => {
      const reply = JSON.parse(String(message.data)) as Reply
      this.pending.get(reply.id)?.(reply)
      this.pending.delete(reply.id)
    })
  }

  static async open(webSocketDebuggerUrl: string): Promise<BareSession> {
    const socket = new WebSocket(webSocketDebuggerUrl)
    await new Promise<void>((resolve, reject) => {
      socket.addEventListener('open', () => resolve())
      socket.addEventListener('error', () =>
        reject(new Error(`could not open ${webSocketDebuggerUrl}`))
      )
    })
    return new BareSession(socket)
  }

  /**
   * `Runtime.evaluate`, with a user gesture so the expression may
   * `window.open`, awaiting a promise it returns.
   */
  async evaluate(expression: string): Promise<unknown> {
    const id = this.nextId++
    const reply = await new Promise<Reply>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id)
        reject(new Error(`Runtime.evaluate did not answer within 15s`))
      }, 15_000)
      this.pending.set(id, (reply) => {
        clearTimeout(timer)
        resolve(reply)
      })
      this.socket.send(
        JSON.stringify({
          id,
          method: 'Runtime.evaluate',
          params: {
            expression,
            userGesture: true,
            awaitPromise: true,
            returnByValue: true
          }
        })
      )
    })
    if (reply.error) {
      throw new Error(reply.error.message)
    }
    if (reply.result?.exceptionDetails) {
      throw new Error(
        `the expression threw: ${JSON.stringify(reply.result.exceptionDetails)}`
      )
    }
    return reply.result?.result.value
  }

  close(): void {
    this.socket.close()
  }
}

/** Polls Chromium's DevTools HTTP endpoint until it answers. */
async function waitForDevToolsServer(
  port: number,
  exited: () => boolean,
  timeout = 60_000
): Promise<void> {
  const deadline = Date.now() + timeout
  for (;;) {
    if (exited()) {
      throw new Error('the app exited before its DevTools server came up')
    }
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/version`)
      if (response.ok) {
        return
      }
    } catch {
      // not listening yet
    }
    if (Date.now() > deadline) {
      throw new Error(
        `the DevTools server never answered on port ${port} within ${timeout}ms`
      )
    }
    await sleep(100)
  }
}

/** Ends the app: politely first, then not. */
async function terminate(child: ChildProcess): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) {
    return
  }
  const exited = new Promise<void>((resolve) =>
    child.once('exit', () => resolve())
  )
  if (process.platform === 'win32') {
    // Takes the helper processes down with it.
    spawnSync('taskkill', ['/pid', String(child.pid), '/T', '/F'], {
      stdio: 'ignore'
    })
  } else {
    child.kill('SIGTERM')
  }
  await Promise.race([
    exited,
    sleep(5_000).then(() => {
      child.kill('SIGKILL')
      return exited
    })
  ])
}

/** A TCP port nothing is listening on right now, above the range CEF refuses. */
function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer()
    server.unref()
    server.on('error', reject)
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address() as net.AddressInfo
      server.close(() => resolve(port))
    })
  })
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
