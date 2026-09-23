// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Shared WebdriverIO configuration for the mobile suites (`wdio.android.conf.ts`
// and `wdio.ios.conf.ts`). Instead of tauri-driver, the app is driven through
// Appium: the UiAutomator2 driver (Android; chromedriver attaches to the
// WebView) or the XCUITest driver (iOS simulator; WebKit remote inspector).
// Debug builds enable webview debugging on both platforms, which is what makes
// the `WEBVIEW_*` context — and `browser.executeAsync` inside it — available.

import path from 'node:path'
import fs from 'node:fs'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

export type MobilePlatform = 'android' | 'ios'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(dirname, '..', '..')
const appDir = path.join(repoRoot, 'examples', 'api')
const tauriDir = path.join(appDir, 'src-tauri')

/** `identifier` in examples/api's tauri.conf.json. */
const appId = 'com.tauri.api'

/** Where Appium looks for drivers; they are devDependencies of this package. */
process.env.APPIUM_HOME ??= dirname

export function mobileConfig(platform: MobilePlatform): WebdriverIO.Config {
  const ios = platform === 'ios' ? iosTarget() : undefined

  // Path passed to the driver as `appium:app`.
  const application =
    process.env.E2E_APP_PATH
    ?? (ios
      ? path.join(
          tauriDir,
          'gen',
          'apple',
          'build',
          ios.outputArch,
          'Tauri API.app'
        )
      : path.join(
          tauriDir,
          'gen',
          'android',
          'app',
          'build',
          'outputs',
          'apk',
          'universal',
          'debug',
          'app-universal-debug.apk'
        ))

  return {
    specs: ['./test/specs/**/*.spec.ts'],
    // One device/simulator, one app instance: the suite runs serially.
    maxInstances: 1,
    capabilities: [
      platform === 'android'
        ? androidCapabilities(application)
        : iosCapabilities(application)
    ],
    services: [
      [
        'appium',
        {
          args: {
            address: '127.0.0.1',
            // Appium 3 requires insecure features to be scoped to a driver.
            // chromedriver_autodownload lets the UiAutomator2 driver fetch a
            // chromedriver matching the device's WebView (see E2E_CHROMEDRIVER).
            allowInsecure: 'uiautomator2:chromedriver_autodownload'
          },
          // First-session setup (chromedriver download, WebDriverAgent build)
          // can be slow, hence the generous timeout.
          appiumStartTimeout: 120_000,
          logPath: path.join(dirname, 'logs')
        }
      ]
    ],
    reporters: ['spec'],
    framework: 'mocha',
    mochaOpts: {
      ui: 'bdd',
      timeout: 120000
    },
    connectionRetryCount: 0,
    // The first session boots the simulator/emulator, installs the app and, on
    // iOS, compiles WebDriverAgent — well over the 120s default that the
    // request to create the session would otherwise be cut at (the driver
    // timeouts in the capabilities below are what actually bound it).
    connectionRetryTimeout: 600_000,
    specFileRetries: Number(process.env.E2E_SPEC_RETRIES ?? 0),
    // Tells the specs (which run in worker processes) what the app runs on;
    // `process.platform` there is the host. See `platform` in test/helpers.
    runnerEnv: { E2E_PLATFORM: platform },

    onPrepare: async () => {
      if (
        !fs.existsSync(
          path.join(repoRoot, 'packages', 'api', 'dist', 'index.js')
        )
      ) {
        throw new Error(
          'packages/api/dist is missing — run `pnpm build:api` at the repo root before the e2e suite.'
        )
      }

      if (!process.env.E2E_SKIP_BUILD && !process.env.E2E_APP_PATH) {
        // `examples/api`'s `tauri` script is `node ../../packages/cli/tauri.js`,
        // which requires the native CLI to be built (`pnpm build:cli`).
        //
        // The generated Android Studio / Xcode project lives in `src-tauri/gen`
        // (gitignored), so it is initialized on a fresh checkout. The build
        // installs the Rust target it needs itself, so init skips that.
        if (
          !fs.existsSync(path.join(tauriDir, 'gen', ios ? 'apple' : 'android'))
        ) {
          tauriCli([platform, 'init', '--ci', '--skip-targets-install'])
        }
        // A debug build, so wry turns on webview debugging (Android
        // `setWebContentsDebuggingEnabled`, iOS `isInspectable`), which is what
        // lets Appium reach the page. Only the target that the device/emulator
        // actually runs is compiled.
        tauriCli(
          ios
            ? [
                'ios',
                'build',
                '--debug',
                '--target',
                ios.name,
                // Simulator builds are not code signed (see
                // `additionalWebviewBundleIds` in `iosCapabilities`).
                '--no-sign'
              ]
            : [
                'android',
                'build',
                '--debug',
                '--apk',
                '--target',
                androidTarget()
              ]
        )
      }

      if (!fs.existsSync(application)) {
        throw new Error(
          `app not found at ${application} — build it (unset E2E_SKIP_BUILD) or point E2E_APP_PATH at an existing build.`
        )
      }
    },

    // The session starts in the native (`NATIVE_APP`) context. Every spec goes
    // through `window.__TAURI__`, so switch to the app's webview as soon as it
    // is attachable and then block until the page has loaded (as the desktop
    // config does).
    before: async (_capabilities, _specs, browser: WebdriverIO.Browser) => {
      let webview: string | undefined
      await browser.waitUntil(
        async () => {
          const contexts = (await browser.getAppiumContexts())
            // Detailed objects are only returned with `appium:fullContextList`.
            .map((context) =>
              typeof context === 'string' ? context : context.id
            )
          // Android names the context after the package, and lists every
          // debuggable WebView on the device (other apps included), so match
          // ours exactly. iOS names it `WEBVIEW_<pid>.<n>` and only lists the
          // app under test's webviews, so the first one is it.
          webview =
            contexts.find((name) => name === `WEBVIEW_${appId}`)
            ?? (platform === 'ios'
              ? contexts.find((name) => name.startsWith('WEBVIEW_'))
              : undefined)
          return webview !== undefined
        },
        {
          // Covers a cold app start plus, on Android, the on-demand chromedriver
          // download for the first session.
          timeout: 120_000,
          interval: 1000,
          timeoutMsg:
            'no WEBVIEW context appeared — is the app a debug build (webview debugging enabled)?'
        }
      )
      await browser.switchAppiumContext(webview!)
      // The specs run the page through `executeAsync`, and the XCUITest driver
      // starts with a script timeout of 0 (every async script times out at
      // once) rather than the 30s the other drivers default to.
      await browser.setTimeout({ script: 30_000 })

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
    }
  }
}

/** Runs `pnpm tauri <args>` in examples/api, failing loudly. */
function tauriCli(args: string[]): void {
  const result = spawnSync('pnpm', ['tauri', ...args], {
    cwd: appDir,
    stdio: 'inherit',
    shell: true
  })
  if (result.status !== 0) {
    throw new Error(
      `\`pnpm tauri ${args.join(' ')}\` failed with status ${result.status}`
    )
  }
}

// --- Android -----------------------------------------------------------------

function androidCapabilities(app: string): WebdriverIO.Capabilities {
  return {
    platformName: 'Android',
    'appium:automationName': 'UiAutomator2',
    'appium:app': app,
    'appium:appPackage': appId,
    'appium:appActivity': '.MainActivity',
    // Which device/emulator to use; Appium picks the first connected one
    // otherwise. E2E_ANDROID_AVD instead boots that AVD.
    ...(process.env.E2E_ANDROID_DEVICE
      ? { 'appium:udid': process.env.E2E_ANDROID_DEVICE }
      : {}),
    ...(process.env.E2E_ANDROID_AVD
      ? { 'appium:avd': process.env.E2E_ANDROID_AVD }
      : {}),
    // chromedriver must match the WebView's Chrome version: either a specific
    // binary, or one Appium downloads on demand.
    ...(process.env.E2E_CHROMEDRIVER
      ? { 'appium:chromedriverExecutable': process.env.E2E_CHROMEDRIVER }
      : { 'appium:chromedriverAutodownload': true }),
    'appium:autoGrantPermissions': true,
    // Every build keeps the same version, so without this Appium sees the app
    // as already installed and runs the previous build instead of this one.
    'appium:enforceAppInstall': true,
    // Emulators in CI are slow; give the UiAutomator2 server and adb room.
    'appium:uiautomator2ServerInstallTimeout': 120_000,
    'appium:uiautomator2ServerLaunchTimeout': 120_000,
    'appium:adbExecTimeout': 60_000,
    'appium:newCommandTimeout': 300
  } as WebdriverIO.Capabilities
}

const androidTargets: Record<string, string> = {
  // `ro.product.cpu.abi` -> `tauri android build --target`
  'arm64-v8a': 'aarch64',
  'armeabi-v7a': 'armv7',
  x86_64: 'x86_64',
  x86: 'i686'
}

/**
 * The Rust target to build the APK for: `E2E_ANDROID_TARGET`, else the ABI of
 * the connected device/emulator (via adb), else the host's (emulators run the
 * host architecture).
 */
function androidTarget(): string {
  if (process.env.E2E_ANDROID_TARGET) {
    return process.env.E2E_ANDROID_TARGET
  }
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const adb = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  const abi = spawnSync(
    adb,
    [
      ...(process.env.E2E_ANDROID_DEVICE
        ? ['-s', process.env.E2E_ANDROID_DEVICE]
        : []),
      'shell',
      'getprop',
      'ro.product.cpu.abi'
    ],
    { encoding: 'utf8', timeout: 10_000 }
  )
  const detected =
    abi.status === 0 ? androidTargets[abi.stdout.trim()] : undefined
  return detected ?? (process.arch === 'arm64' ? 'aarch64' : 'x86_64')
}

// --- iOS ---------------------------------------------------------------------

function iosCapabilities(app: string): WebdriverIO.Capabilities {
  return {
    platformName: 'iOS',
    'appium:automationName': 'XCUITest',
    'appium:app': app,
    'appium:bundleId': appId,
    // The driver looks the app up in the simulator's Web Inspector listing by
    // bundle identifier. The inspector identifies an app by the
    // `application-identifier` entitlement Xcode embeds when it code signs a
    // simulator build (`__TEXT,__entitlements`); `--no-sign` skips signing
    // altogether (`CODE_SIGNING_ALLOWED=NO`), so the entitlement is missing and
    // the inspector falls back to `process-<executable name>`. Match that too.
    'appium:additionalWebviewBundleIds': [
      `process-${path.basename(app, '.app')}`
    ],
    'appium:udid': iosSimulator(),
    // As on Android: the version never changes between builds.
    'appium:enforceAppInstall': true,
    // No Simulator.app window. Besides not needing one, the driver otherwise
    // shuts a simulator that is booted without a visible UI down to relaunch it
    // with one, on every session — and that shutdown regularly outlasts the
    // driver's 15s limit on it.
    'appium:isHeadless': true,
    // A prebuilt WebDriverAgent (Appium's release build for the simulator, see
    // `appium driver run xcuitest download-wda -- --kind sim`) is installed and
    // launched as is, instead of being compiled with xcodebuild on the first
    // session, which takes minutes on a CI runner.
    ...(process.env.E2E_IOS_WDA
      ? {
          'appium:usePreinstalledWDA': true,
          'appium:prebuiltWDAPath': process.env.E2E_IOS_WDA
        }
      : {}),
    // Without one, WebDriverAgent is compiled on the first session, which takes
    // minutes on a CI runner.
    'appium:wdaLaunchTimeout': 240_000,
    'appium:wdaStartupRetries': 3,
    'appium:simulatorStartupTimeout': 240_000,
    'appium:newCommandTimeout': 300
  } as WebdriverIO.Capabilities
}

/**
 * The Rust target to build the simulator app for (`E2E_IOS_TARGET`, else the
 * host's architecture) and the directory the CLI exports it to.
 */
function iosTarget(): { name: string; outputArch: string } {
  const name =
    process.env.E2E_IOS_TARGET
    ?? (process.arch === 'arm64' ? 'aarch64-sim' : 'x86_64')
  // `tauri ios build` writes to gen/apple/build/<arch>/ (cargo-mobile2's
  // `arch` for the target).
  const outputArch = name === 'aarch64-sim' ? 'arm64-sim' : name
  return { name, outputArch }
}

interface SimctlDevice {
  name: string
  udid: string
  state: string
  isAvailable: boolean
  deviceTypeIdentifier?: string
}

/**
 * UDID of the simulator to run on: `E2E_IOS_DEVICE` (a UDID or device name),
 * else an already-booted iPhone, else the iPhone on the newest installed
 * runtime. Resolved through `simctl` so nothing has to be hardcoded per Xcode
 * version.
 */
function iosSimulator(): string {
  const requested = process.env.E2E_IOS_DEVICE
  if (requested && /^[0-9A-F-]{36}$/i.test(requested)) {
    return requested
  }
  const list = spawnSync(
    'xcrun',
    ['simctl', 'list', 'devices', 'available', '--json'],
    { encoding: 'utf8' }
  )
  if (list.status !== 0) {
    throw new Error(
      `\`xcrun simctl list\` failed — is Xcode installed?\n${list.stderr}`
    )
  }
  const runtimes = JSON.parse(list.stdout).devices as Record<
    string,
    SimctlDevice[]
  >
  const iphones = Object.entries(runtimes)
    // e.g. `com.apple.CoreSimulator.SimRuntime.iOS-18-2`
    .filter(([runtime]) => runtime.includes('.iOS-'))
    .sort(([a], [b]) => runtimeVersion(b) - runtimeVersion(a))
    .flatMap(([, devices]) =>
      devices.filter(
        (device) =>
          device.isAvailable
          && (device.deviceTypeIdentifier ?? device.name).includes('iPhone')
      )
    )
  const device = requested
    ? iphones.find((candidate) => candidate.name === requested)
    : (iphones.find((candidate) => candidate.state === 'Booted') ?? iphones[0])
  if (!device) {
    throw new Error(
      requested
        ? `no available iPhone simulator named "${requested}" (E2E_IOS_DEVICE)`
        : 'no available iPhone simulator found — install an iOS runtime in Xcode.'
    )
  }
  return device.udid
}

function runtimeVersion(runtime: string): number {
  const [major = '0', minor = '0'] = (runtime.split('.iOS-')[1] ?? '').split(
    '-'
  )
  return Number(major) * 100 + Number(minor)
}
