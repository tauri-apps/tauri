# `@tauri-apps/api` end-to-end tests

WebdriverIO suite that exercises every [`@tauri-apps/api`](../api) module against a
**real** Tauri app — the [`examples/api`](../../examples/api) validation app — rather than
a mocked backend, on desktop (Linux, macOS, Windows) and mobile (Android, iOS). Each module
has its own spec file, shared by every platform, and adding coverage for a new API is
normally just dropping in one more spec.

## How it works

- The example app is built with `withGlobalTauri: true`, so the entire API surface is
  reachable on `window.__TAURI__` inside the webview.
- On desktop, WebdriverIO drives the app through [`@crabnebula/tauri-driver`](https://www.npmjs.com/package/@crabnebula/tauri-driver),
  which bridges the WebDriver protocol to each platform's webview:
  - **macOS** — the CrabNebula Webdriver, which needs [`tauri-plugin-automation`](https://crates.io/crates/tauri-plugin-automation)
    (registered in `examples/api` behind its off-by-default `automation` Cargo feature, which
    the suite's build enables) and a locally-running `@crabnebula/test-runner-backend`,
    authenticated with `CN_API_KEY`.
  - **Linux** — `webkit2gtk-driver` (`WebKitWebDriver` on `PATH`).
  - **Windows** — `msedgedriver.exe` on `PATH`. It hands the app the `--remote-debugging-port`
    it attaches to through `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, which WebView2 ignores in an
    elevated process ([wry#1782](https://github.com/tauri-apps/wry/issues/1782)), so the suite has
    to run unelevated.
- On mobile, WebdriverIO drives the app through [Appium](https://appium.io) (started by
  `@wdio/appium-service`; the drivers are plain devDependencies of this package, which Appium
  picks up on its own):
  - **Android** — the UiAutomator2 driver. The suite switches to the app's `WEBVIEW_*`
    context, which chromedriver reaches through the WebView's debugging socket. Debug builds
    turn that on (`setWebContentsDebuggingEnabled`), so the suite builds a debug APK. A
    chromedriver matching the device's WebView is downloaded on demand (see `E2E_CHROMEDRIVER`).
  - **iOS** — the XCUITest driver on a simulator, attaching to the WKWebView through the
    WebKit remote inspector. Debug builds mark the webview `isInspectable`, so the suite
    builds an unsigned debug simulator app. The inspector identifies an app by the
    `application-identifier` entitlement that Xcode embeds when it code signs a simulator
    build; an unsigned one has none and is listed as `process-<executable name>` instead of
    its bundle identifier, so the config has the driver match that name too
    (`appium:additionalWebviewBundleIds`). The driver also starts with a script timeout of
    0, which the config raises to the 30s the other drivers default to, or every
    `executeAsync` would time out at once.
- Specs never `eval` in the page. They pass a function to the [`tauri()`](test/helpers/index.ts)
  helper, which serializes it and runs it via the driver's own (CSP-exempt) script injection,
  handing it `window.__TAURI__` as the first argument and returning its JSON result.
- Each spec file gets its own session — a fresh `tauri-driver` (and therefore a fresh app
  instance) on desktop, a fresh Appium session (which relaunches the app) on mobile — so
  each module's suite runs in isolation.

## Prerequisites

```sh
# from the repo root
pnpm install
pnpm build:api    # examples/api resolves @tauri-apps/api from packages/api/dist
pnpm build:cli    # examples/api's `tauri` script uses the local native CLI
```

Platform driver dependencies:

| Platform | Requirement                                                                                                                                                                                                        |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS    | `CN_API_KEY` env var (CrabNebula Cloud). The automation plugin and test-runner-backend are wired up already.                                                                                                       |
| Linux    | `webkit2gtk-driver` package (provides `WebKitWebDriver`).                                                                                                                                                          |
| Windows  | `msedgedriver.exe` matching your Edge version, on `PATH`. Run the suite unelevated.                                                                                                                                |
| Android  | The usual Tauri Android setup (`ANDROID_HOME`, `NDK_HOME`, a JDK), plus a running emulator or a connected device with USB debugging. Network access the first time, for the chromedriver download.                 |
| iOS      | macOS with Xcode and an iOS simulator runtime. `tauri ios init` installs [xcodegen](https://github.com/yonaskolb/XcodeGen) through Homebrew if missing. The first session compiles WebDriverAgent (a few minutes). |

## Running

```sh
# from the repo root
pnpm test:api-e2e

# or from this package
pnpm e2e

# iterate without rebuilding the app every run
E2E_SKIP_BUILD=1 pnpm e2e

# run a single module's spec
pnpm exec wdio run ./wdio.conf.ts --spec test/specs/window.spec.ts

# mobile (from the repo root; or `pnpm e2e:android` / `pnpm e2e:ios` from this package)
pnpm test:api-e2e:android
pnpm test:api-e2e:ios
```

The first run builds the app with [`tauri.e2e.conf.json`](tauri.e2e.conf.json) as a config
override, which enables the example's `automation` feature; afterwards use `E2E_SKIP_BUILD=1`
to reuse the existing binary. A binary supplied through `E2E_SKIP_BUILD` or `E2E_APP_PATH`
must have been built with that feature whenever the CrabNebula Webdriver is in use (always
on macOS).

The mobile configs ([`wdio.android.conf.ts`](wdio.android.conf.ts), [`wdio.ios.conf.ts`](wdio.ios.conf.ts),
sharing [`wdio.mobile.ts`](wdio.mobile.ts)) initialize the example's Android/Xcode project if
`src-tauri/gen` is missing, then run `tauri android build --debug --apk` /
`tauri ios build --debug --target aarch64-sim --no-sign`, compiling only the Rust target the
device runs (the Android one is read from the connected device through `adb`). The app must be
a **debug** build — release builds have webview debugging off, and Appium cannot see the page.
`E2E_SKIP_BUILD` and `E2E_APP_PATH` (an `.apk` / simulator `.app`) work as on desktop.

The generated Gradle and Xcode projects call back into the CLI with `pnpm tauri …` from
`src-tauri` / `gen/apple`, which pnpm 12.0–12.3 could not resolve to the package's scripts
([pnpm/pnpm#14645](https://github.com/pnpm/pnpm/pull/14645)); the repo's `packageManager` pins a
fixed version, so run the suite through that pnpm (corepack) rather than an older global one.

## Environment variables

| Variable            | Purpose                                                                           |
| ------------------- | --------------------------------------------------------------------------------- |
| `CN_API_KEY`        | CrabNebula Cloud key. Required on macOS (and whenever `E2E_CN_WEBDRIVER=1`).      |
| `E2E_SKIP_BUILD`    | Skip the `tauri build` step and reuse the existing binary.                        |
| `E2E_APP_PATH`      | Absolute path to a prebuilt app/binary to test (also implies skip-build).         |
| `E2E_SKIP`          | Comma-separated module names to skip, e.g. `E2E_SKIP=tray,menu`.                  |
| `E2E_SKIP_WM`       | Skip window-manager-dependent tests (minimize/maximize/fullscreen/position/hide). |
| `E2E_SPEC_RETRIES`  | Retry count for flaky spec files (default `0`).                                   |
| `E2E_CN_WEBDRIVER`  | Use the CrabNebula Webdriver on Linux/Windows too (instead of the native driver). |
| `E2E_NATIVE_DRIVER` | Path passed to `tauri-driver --native-driver` (e.g. a specific chromedriver).     |
| `CARGO_TARGET_DIR`  | Override the target dir the app binary is looked up in.                           |

Mobile only:

| Variable             | Purpose                                                                                              |
| -------------------- | ---------------------------------------------------------------------------------------------------- |
| `E2E_ANDROID_TARGET` | Rust target for the APK (`aarch64`, `armv7`, `i686`, `x86_64`); default: the connected device's ABI. |
| `E2E_ANDROID_DEVICE` | `adb` serial of the device/emulator to use (`appium:udid`); default: the first connected one.        |
| `E2E_ANDROID_AVD`    | Name of an AVD for Appium to boot (`appium:avd`) instead of using an already-running emulator.       |
| `E2E_CHROMEDRIVER`   | chromedriver binary matching the device's WebView, instead of letting Appium download one.           |
| `E2E_IOS_TARGET`     | Rust target for the simulator app (`aarch64-sim` or `x86_64`); default: the host architecture.       |
| `E2E_IOS_DEVICE`     | Simulator UDID or name (as in `xcrun simctl list`); default: a booted iPhone, else the newest one.   |
| `E2E_PLATFORM`       | Set by the mobile configs for the spec workers (`android`/`ios`) — see `platform` in the helpers.    |

Appium's own log is written to `logs/wdio-appium.log` in this package.

## Adding tests for a new API

1. **Add a spec.** Create `test/specs/<module>.spec.ts` and use `describeApi('<module>', …)`
   with the `tauri()` helper. It is picked up automatically by the `test/specs/**/*.spec.ts`
   glob. Minimal example:

   ```ts
   import { expect } from '@wdio/globals'
   import { tauri, describeApi } from '../helpers/index.js'

   describeApi('app', () => {
     it('reports the product name', async () => {
       expect(await tauri((api) => api.app.getName())).toBe('Tauri API')
     })
   })
   ```

2. **Grant permissions if needed.** If the API calls a command that is not allowed by
   default, add the permission to
   [`examples/api/src-tauri/capabilities/run-app.json`](../../examples/api/src-tauri/capabilities/run-app.json).
   The default permission sets are documented under
   `crates/tauri/permissions/*/autogenerated/reference.md`.

3. **Add a Rust command if needed.** If the API needs a custom backend command, add it to
   [`examples/api/src-tauri/src/cmd.rs`](../../examples/api/src-tauri/src/cmd.rs), register it
   in the `generate_handler!` list in
   [`src/lib.rs`](../../examples/api/src-tauri/src/lib.rs), add its permission, and grant it
   in `run-app.json`.

4. **Handle environment-sensitive cases.** Use `itWm` (instead of `it`) for assertions that
   depend on a real window manager, and `eventually()` to poll for state a WM applies
   asynchronously. Branch on `platform` from the helpers (never `process.platform`, which is
   the host running the emulator/simulator on mobile) for platform-specific behavior.

5. **Gate what mobile does not have.** The same specs run on Android and iOS. Wrap tests of
   desktop-only commands (`#[cfg(desktop)]` in the core plugins, or no-ops on mobile such as
   the window title and size) in `itDesktop`, use `itOn('android', …)` / `itOn('ios', …)` for
   platform-specific APIs, and pass `{ desktopOnly: true }` to `describeApi` for modules whose
   plugin is not registered on mobile at all (`menu`, `tray`). Skipped tests show up as pending
   rather than silently disappearing.

### Rules for `tauri()` page functions

The function you pass to `tauri()` runs **inside the webview**, serialized as a string:

- It **cannot** close over anything from the spec module — pass every value it needs through
  the trailing `tauri(fn, ...args)` arguments.
- It may only reference `api` (the `window.__TAURI__` object), those args, and browser
  globals (`window`, `document`, `setTimeout`, `Promise`, …).
- Its return value must be JSON-serializable — return plain objects/primitives, not class
  instances (call methods and return their results instead).
- Restore any app state you mutate (title, size, theme, …); tests within a spec file share
  the same app instance.
- For in-page waiting, wrap logic in a `Promise` with an explicit `setTimeout` rejection so a
  failure surfaces as a message rather than an opaque driver timeout.

Use `tauriError(fn, ...args)` to assert that a call rejects; it returns the rejection message.
