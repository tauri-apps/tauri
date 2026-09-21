// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import fs from 'node:fs'
import { spawn, spawnSync, type ChildProcess } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { SevereServiceError } from 'webdriverio'
import { waitTauriDriverReady } from '@crabnebula/tauri-driver'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(dirname, '..', '..')
const appDir = path.join(repoRoot, 'examples', 'api')
const targetDir = process.env.CARGO_TARGET_DIR ?? path.join(repoRoot, 'target')

// The webview runtime the app is built with: `wry` (the system webview, the
// example's default) or `cef` (Chromium Embedded Framework, the example's `cef`
// feature). Both builds land at the same output path, so a binary reused through
// E2E_SKIP_BUILD must have been built for the selected runtime.
const runtime = process.env.E2E_RUNTIME ?? 'wry'
if (runtime !== 'wry' && runtime !== 'cef') {
  throw new Error(`E2E_RUNTIME must be "wry" or "cef", got "${runtime}"`)
}

// macOS has no native WebDriver for WKWebView, so the CrabNebula Webdriver
// (backed by tauri-plugin-automation + the test-runner-backend) is required there.
// The native drivers tauri-driver spawns elsewhere (WebKitWebDriver, msedgedriver)
// only speak to the system webview, so a CEF app needs it on every platform: the
// automation plugin drives the app through Tauri's own IPC and is runtime-agnostic.
// It can be opted into for wry on Linux/Windows via E2E_CN_WEBDRIVER for parity.
const useCrabNebulaWebdriver =
  process.platform === 'darwin'
  || runtime === 'cef'
  || !!process.env.E2E_CN_WEBDRIVER

// Path passed to the driver as `tauri:options.application`.
const application =
  process.env.E2E_APP_PATH
  ?? (process.platform === 'darwin'
    ? path.join(targetDir, 'debug', 'bundle', 'macos', 'Tauri API.app')
    : process.platform === 'win32'
      ? path.join(targetDir, 'debug', 'api.exe')
      : path.join(targetDir, 'debug', 'api'))

let tauriDriver: ChildProcess | undefined
let killedTauriDriver = false
let testRunnerBackend: ChildProcess | undefined
let killedTestRunnerBackend = false

export const config: WebdriverIO.Config = {
  hostname: '127.0.0.1',
  port: 4444,
  specs: ['./test/specs/**/*.spec.ts'],
  // The driver spawns a single app instance and speaks WebDriver to it, so the
  // suite must run serially.
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      // `tauri:options` is understood by tauri-driver, not by the WebdriverIO types.
      'tauri:options': { application }
    } as unknown as WebdriverIO.Capabilities
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000
  },
  connectionRetryCount: 0,
  specFileRetries: Number(process.env.E2E_SPEC_RETRIES ?? 0),

  // Every failure here is a `SevereServiceError`: wdio merely logs a plain error
  // thrown by a launcher hook and starts the workers anyway, which would then
  // drive whatever binary happens to sit at the output path (or time out on a
  // missing one) instead of failing on the actual cause.
  onPrepare: async () => {
    // The example resolves `@tauri-apps/api` from `packages/api/dist`, and its
    // frontend build (vite) needs it too. Fail early with a clear message.
    if (
      !fs.existsSync(path.join(repoRoot, 'packages', 'api', 'dist', 'index.js'))
    ) {
      throw new SevereServiceError(
        'packages/api/dist is missing — run `pnpm build:api` at the repo root before the e2e suite.'
      )
    }

    if (!process.env.E2E_SKIP_BUILD && !process.env.E2E_APP_PATH) {
      // `examples/api`'s `tauri` script is `node ../../packages/cli/tauri.js`,
      // which requires the native CLI to be built (`pnpm build:cli`).
      //
      // The override config does two things for this (debug, test-only) build:
      //  - enables the example's off-by-default `automation` feature, which
      //    registers tauri-plugin-automation;
      //  - disables `removeUnusedCommands`, which the example otherwise sets.
      //    It strips command handlers that no static capability grants, and the
      //    plugin adds its `resolve` command through a *runtime* capability, so
      //    it would be stripped — breaking the CrabNebula Webdriver on macOS.
      // Passed as an appDir-relative path to sidestep shell quoting.
      const overrideConfig = path.relative(
        appDir,
        path.join(dirname, 'tauri.e2e.conf.json')
      )
      const buildArgs = [
        'tauri',
        'build',
        '--debug',
        '--config',
        overrideConfig,
        ...(process.platform === 'darwin'
          ? ['--bundles', 'app'] // the .app bundle is needed for tauri:options
          : ['--no-bundle']),
        // The example picks its runtime through Cargo features and defaults to
        // `wry`, so CEF needs the default set off as well. `--no-default-features`
        // is a cargo flag, which the CLI forwards from after `--`. The CLI detects
        // the runtime from the enabled features and ships the CEF framework and
        // helper apps in the macOS bundle; on Linux/Windows the cef build script
        // lays the distribution out next to the bare binary in the target dir.
        ...(runtime === 'cef'
          ? ['--features', 'cef', '--', '--no-default-features']
          : [])
      ]
      const build = spawnSync('pnpm', buildArgs, {
        cwd: appDir,
        stdio: 'inherit',
        shell: true
      })
      if (build.status !== 0) {
        throw new SevereServiceError(
          `\`pnpm ${buildArgs.join(' ')}\` failed with status ${build.status}`
        )
      }
    }

    if (!fs.existsSync(application)) {
      throw new SevereServiceError(
        `app not found at ${application} — build it (unset E2E_SKIP_BUILD) or point E2E_APP_PATH at an existing build.`
      )
    }

    // A CEF app on macOS only runs from a bundle that carries the framework: the
    // executable aborts loading it otherwise. A stale CLI build (one predating the
    // bundler's runtime detection) produces exactly such a bundle without failing,
    // so catch it here rather than as an opaque driver timeout.
    if (
      runtime === 'cef'
      && process.platform === 'darwin'
      && !fs.existsSync(
        path.join(
          application,
          'Contents',
          'Frameworks',
          'Chromium Embedded Framework.framework'
        )
      )
    ) {
      throw new SevereServiceError(
        `${application} does not bundle the CEF framework — it was not built for the CEF runtime (or by a CLI that predates it: run \`pnpm build:cli\` and rebuild).`
      )
    }

    if (useCrabNebulaWebdriver) {
      if (!process.env.CN_API_KEY) {
        throw new SevereServiceError(
          'CN_API_KEY is required for the CrabNebula Webdriver (mandatory on macOS and for E2E_RUNTIME=cef, or when E2E_CN_WEBDRIVER=1).'
        )
      }
      testRunnerBackend = spawn('pnpm', ['exec', 'test-runner-backend'], {
        cwd: dirname,
        stdio: 'inherit',
        shell: true,
        // Lead a new process group so the whole tree (sh -> pnpm ->
        // test-runner-backend) can be torn down together. See killProcessTree.
        detached: process.platform !== 'win32'
      })
      testRunnerBackend.on('error', (error) => {
        console.error('test-runner-backend error:', error)
        process.exit(1)
      })
      testRunnerBackend.on('exit', (code) => {
        if (!killedTestRunnerBackend) {
          console.error('test-runner-backend exited with code:', code)
          process.exit(1)
        }
      })
      const { waitTestRunnerBackendReady } =
        await import('@crabnebula/test-runner-backend')
      await waitTestRunnerBackendReady()
      process.env.REMOTE_WEBDRIVER_URL = 'http://127.0.0.1:3000'
    }
  },

  // A fresh tauri-driver (and therefore a fresh app instance) per spec file,
  // so each module's suite runs in isolation.
  beforeSession: async () => {
    const args = ['exec', 'tauri-driver']
    if (process.env.E2E_NATIVE_DRIVER) {
      args.push('--native-driver', process.env.E2E_NATIVE_DRIVER)
    }
    // Reset before each (re)spawn so the `exit` handler below still treats an
    // unexpected driver crash as fatal on retried spec files.
    killedTauriDriver = false
    tauriDriver = spawn('pnpm', args, {
      cwd: dirname,
      stdio: [null, process.stdout, process.stderr],
      shell: true,
      // Lead a new process group so the whole tree (sh -> pnpm -> tauri-driver
      // -> native webdriver) can be torn down together. See killProcessTree.
      detached: process.platform !== 'win32'
    })
    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error)
      process.exit(1)
    })
    tauriDriver.on('exit', (code) => {
      if (!killedTauriDriver) {
        console.error('tauri-driver exited with code:', code)
        process.exit(1)
      }
    })
    await waitTauriDriverReady()
  },

  // The session is created as soon as the app's window exists, which can be
  // before the webview has navigated to the app's page: WebView2 on a cold start
  // (the first launches on a fresh Windows runner) still shows `about:blank`
  // for a few seconds, so the first spec's scripts ran in a page without
  // `window.__TAURI__`. Every spec goes through that global, so block until
  // it exists.
  before: async (_capabilities, _specs, browser) => {
    await browser.waitUntil(
      async () => {
        try {
          return (await browser.executeAsync(
            'var done = arguments[arguments.length - 1]; done(typeof window.__TAURI__ !== "undefined");'
          )) as boolean
        } catch {
          // A command issued mid-navigation can fail on a stale execution
          // context; that just means "not ready yet".
          return false
        }
      },
      {
        timeout: 30_000,
        interval: 250,
        timeoutMsg:
          'window.__TAURI__ never became available — the app did not load its page.'
      }
    )
  },

  // Awaited so the driver (and its port) is fully gone before the next spec's
  // beforeSession spawns a new one on the same port.
  afterSession: async () => {
    await closeTauriDriver()
  },

  onComplete: () => {
    closeAll()
  }
}

/**
 * Kills a shell-spawned child and everything it started. `child.kill()` only
 * signals the `sh`/`cmd` wrapper, orphaning the real process (which keeps
 * holding the WebDriver port and poisons every later spec), so we take down the
 * whole process group/tree instead.
 */
function killProcessTree(
  child: ChildProcess | undefined,
  signal: NodeJS.Signals = 'SIGTERM'
): void {
  const pid = child?.pid
  if (!pid || child?.exitCode !== null) return
  if (process.platform === 'win32') {
    spawnSync('taskkill', ['/pid', String(pid), '/T', '/F'], {
      stdio: 'ignore'
    })
  } else {
    try {
      // Negative pid targets the whole process group (see `detached` above).
      process.kill(-pid, signal)
    } catch {
      try {
        child!.kill(signal)
      } catch {
        // already gone
      }
    }
  }
}

async function closeTauriDriver(): Promise<void> {
  killedTauriDriver = true
  const driver = tauriDriver
  tauriDriver = undefined
  if (!driver || driver.exitCode !== null) return
  const exited = new Promise<void>((resolve) =>
    driver.once('exit', () => resolve())
  )
  killProcessTree(driver)
  // Give it a moment to release the port, then escalate to SIGKILL.
  let timer: ReturnType<typeof setTimeout> | undefined
  await Promise.race([
    exited,
    new Promise<void>((resolve) => {
      timer = setTimeout(() => {
        killProcessTree(driver, 'SIGKILL')
        resolve()
      }, 5000)
    })
  ])
  if (timer) clearTimeout(timer)
}

function closeAll() {
  killedTauriDriver = true
  killProcessTree(tauriDriver)
  tauriDriver = undefined
  killedTestRunnerBackend = true
  killProcessTree(testRunnerBackend)
  testRunnerBackend = undefined
}

function onShutdown(fn: () => void) {
  const cleanup = () => {
    try {
      fn()
    } finally {
      process.exit()
    }
  }
  for (const signal of [
    'exit',
    'SIGINT',
    'SIGTERM',
    'SIGHUP',
    'SIGBREAK'
  ] as const) {
    process.on(signal, cleanup)
  }
}

onShutdown(closeAll)
