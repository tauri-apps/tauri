# `@tauri-apps/api` end-to-end tests

WebdriverIO suite that exercises every [`@tauri-apps/api`](../api) module against a
**real** Tauri app — the [`examples/api`](../../examples/api) validation app — rather than
a mocked backend. Each module has its own spec file, and adding coverage for a new API is
normally just dropping in one more spec.

## How it works

- The example app is built with `withGlobalTauri: true`, so the entire API surface is
  reachable on `window.__TAURI__` inside the webview.
- The app is built for one of its two webview runtimes, selected with `E2E_RUNTIME`:
  **wry** (the system webview, the default) or **cef** (the Chromium Embedded Framework,
  through the example's `cef` Cargo feature). Both builds land at the same output path.
- WebdriverIO drives the app through [`@crabnebula/tauri-driver`](https://www.npmjs.com/package/@crabnebula/tauri-driver),
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
  - **CEF, on every platform** — the CrabNebula Webdriver. The native drivers above only speak
    to the system webview, whereas the automation plugin reaches the app through Tauri's own IPC
    and works with any runtime. The `automation` feature also switches the CEF runtime to
    `SecretStorage::Mock` (`--use-mock-keychain` on macOS, `--password-store=basic` on Linux):
    the test bundle is ad-hoc signed, so the OS keychain would otherwise prompt on every rebuild
    and never get an answer in CI.
- Specs never `eval` in the page. They pass a function to the [`tauri()`](test/helpers/index.ts)
  helper, which serializes it and runs it via the driver's own (CSP-exempt) script injection,
  handing it `window.__TAURI__` as the first argument and returning its JSON result.
- A fresh `tauri-driver` (and therefore a fresh app instance) is started per spec file, so
  each module's suite runs in isolation.

## Prerequisites

```sh
# from the repo root
pnpm install
pnpm build:api    # examples/api resolves @tauri-apps/api from packages/api/dist
pnpm build:cli    # examples/api's `tauri` script uses the local native CLI
```

Platform driver dependencies:

| Platform      | Requirement                                                                                                               |
| ------------- | ------------------------------------------------------------------------------------------------------------------------- |
| macOS         | `CN_API_KEY` env var (CrabNebula Cloud). The automation plugin and test-runner-backend are wired up already.              |
| Linux (wry)   | `webkit2gtk-driver` package (provides `WebKitWebDriver`).                                                                 |
| Windows (wry) | `msedgedriver.exe` matching your Edge version, on `PATH`. Run the suite unelevated.                                       |
| CEF (any OS)  | `CN_API_KEY` env var, and `libpipewire-0.3` on Linux (the driver links it). The `cef` crate downloads CEF on first build. |

On Linux the app also needs a tray host: it registers a tray icon at startup, which is a
StatusNotifierItem on the session bus, and fails to start when nothing owns
`org.kde.StatusNotifierWatcher` there. Any desktop with a tray provides one; in a headless X
session (Xvfb with a bare window manager, as in CI) run
[`.scripts/ci/sni-watcher.py`](../../.scripts/ci/sni-watcher.py) (needs `python3-gi`) on the
session bus the app will use — see the workflow for the `dbus-run-session` incantation.

For CEF, the first build also downloads CEF (~150 MB) into the CLI's cache dir (or `CEF_PATH`)
and, on macOS, compiles the helper apps the bundler ships in the `.app` (a few minutes, cached
afterwards). On Linux a `--no-bundle` build runs Chromium's sandbox through unprivileged user
namespaces, which Ubuntu 24.04 restricts by default
(`sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0` lifts it), since the
`chrome-sandbox` helper next to the binary only gets its setuid bit from the deb/rpm installers.

## Running

```sh
# from the repo root
pnpm test:api-e2e

# or from this package
pnpm e2e

# iterate without rebuilding the app every run
E2E_SKIP_BUILD=1 pnpm e2e

# the same suite against the CEF runtime (needs CN_API_KEY on every platform)
pnpm test:api-e2e:cef   # from the repo root
pnpm e2e:cef            # from this package
pnpm e2e:cef:skip-build

# run a single module's spec
pnpm exec wdio run ./wdio.conf.ts --spec test/specs/window.spec.ts
```

The first run builds the app with [`tauri.e2e.conf.json`](tauri.e2e.conf.json) as a config
override, which enables the example's `automation` feature (`E2E_RUNTIME=cef` adds
`--features cef -- --no-default-features` to swap wry out for the CEF runtime); afterwards use
`E2E_SKIP_BUILD=1` to reuse the existing binary. A binary supplied through `E2E_SKIP_BUILD` or
`E2E_APP_PATH` must have been built with that feature whenever the CrabNebula Webdriver is in
use (always on macOS and for CEF), and for the runtime the suite is being run against — the
wry and CEF builds overwrite each other.

## Environment variables

| Variable            | Purpose                                                                                   |
| ------------------- | ----------------------------------------------------------------------------------------- |
| `E2E_RUNTIME`       | Webview runtime to build and test the app with: `wry` (default) or `cef`.                 |
| `CN_API_KEY`        | CrabNebula Cloud key. Required on macOS, for CEF, and whenever `E2E_CN_WEBDRIVER=1`.      |
| `E2E_SKIP_BUILD`    | Skip the `tauri build` step and reuse the existing binary.                                |
| `E2E_APP_PATH`      | Absolute path to a prebuilt app/binary to test (also implies skip-build).                 |
| `E2E_SKIP`          | Comma-separated module names to skip, e.g. `E2E_SKIP=tray,menu`.                          |
| `E2E_SKIP_WM`       | Skip window-manager-dependent tests (minimize/maximize/fullscreen/position/hide).         |
| `E2E_SPEC_RETRIES`  | Retry count for flaky spec files (default `0`).                                           |
| `E2E_CN_WEBDRIVER`  | Use the CrabNebula Webdriver for wry on Linux/Windows too (instead of the native driver). |
| `E2E_NATIVE_DRIVER` | Path passed to `tauri-driver --native-driver` (e.g. a specific chromedriver).             |
| `CARGO_TARGET_DIR`  | Override the target dir the app binary is looked up in.                                   |
| `CEF_PATH`          | Where the `cef` crate keeps (and downloads) the CEF binary distribution for CEF builds.   |

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
   asynchronously. Branch on `process.platform` (Node side) for platform-specific behavior.

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
