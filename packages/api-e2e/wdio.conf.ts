// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import fs from 'node:fs'
import { spawn, spawnSync, type ChildProcess } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { waitTauriDriverReady } from '@crabnebula/tauri-driver'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(dirname, '..', '..')
const appDir = path.join(repoRoot, 'examples', 'api')
const targetDir = process.env.CARGO_TARGET_DIR ?? path.join(repoRoot, 'target')

// macOS has no native WebDriver for WKWebView, so the CrabNebula Webdriver
// (backed by tauri-plugin-automation + the test-runner-backend) is required there.
// It can be opted into on the other platforms via E2E_CN_WEBDRIVER for parity.
const useCrabNebulaWebdriver =
  process.platform === 'darwin' || !!process.env.E2E_CN_WEBDRIVER

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

  onPrepare: async () => {
    // The example resolves `@tauri-apps/api` from `packages/api/dist`, and its
    // frontend build (vite) needs it too. Fail early with a clear message.
    if (
      !fs.existsSync(path.join(repoRoot, 'packages', 'api', 'dist', 'index.js'))
    ) {
      throw new Error(
        'packages/api/dist is missing — run `pnpm build:api` at the repo root before the e2e suite.'
      )
    }

    if (!process.env.E2E_SKIP_BUILD && !process.env.E2E_APP_PATH) {
      // `examples/api`'s `tauri` script is `node ../../packages/cli/tauri.js`,
      // which requires the native CLI to be built (`pnpm build:cli`).
      //
      // The example sets `removeUnusedCommands: true`, which strips command
      // handlers that no static capability grants. tauri-plugin-automation adds
      // its `resolve` command through a *runtime* capability, so it would be
      // stripped — breaking the CrabNebula Webdriver on macOS. The override
      // config disables that stripping for this (debug, test-only) build.
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
          : ['--no-bundle'])
      ]
      const build = spawnSync('pnpm', buildArgs, {
        cwd: appDir,
        stdio: 'inherit',
        shell: true
      })
      if (build.status !== 0) {
        throw new Error(
          `\`pnpm ${buildArgs.join(' ')}\` failed with status ${build.status}`
        )
      }
    }

    if (!fs.existsSync(application)) {
      throw new Error(
        `app not found at ${application} — build it (unset E2E_SKIP_BUILD) or point E2E_APP_PATH at an existing build.`
      )
    }

    if (useCrabNebulaWebdriver) {
      if (!process.env.CN_API_KEY) {
        throw new Error(
          'CN_API_KEY is required for the CrabNebula Webdriver (mandatory on macOS, or when E2E_CN_WEBDRIVER=1).'
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
