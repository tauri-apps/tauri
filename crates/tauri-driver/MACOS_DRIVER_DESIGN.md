# tauri-driver on macOS — design notes

Status: **scaffold**. Tracks
[tauri-apps/tauri#7068](https://github.com/tauri-apps/tauri/issues/7068).
Nothing here is wired into the runtime path yet; `tauri-driver` on macOS still
exits with a non-zero status and a pointer to this document. The goal of the
initial scaffold PR is to land the platform plumbing (cfg gates, module
layout, CLI parity, unit tests) so the follow-up PR that fills in the bridge
is a small, reviewable diff rather than a cross-cutting rewrite.

## Why this is not already done

`tauri-driver` works by spawning a *native* WebDriver server and then
proxying W3C WebDriver requests to it, rewriting the new-session payload
along the way:

| Platform | Native driver       | Capability key forwarded         |
| -------- | ------------------- | -------------------------------- |
| Linux    | `WebKitWebDriver`   | `webkitgtk:browserOptions`       |
| Windows  | `msedgedriver.exe`  | `ms:edgeOptions`                 |
| macOS    | (none, today)       | (n/a)                            |

The macOS row is empty because Apple does not ship a WebDriver implementation
for `WKWebView`. The closest pieces are:

1. **`safaridriver`** — ships with macOS, drives **Safari**, not `WKWebView`.
   It speaks W3C WebDriver, so the proxy half of `tauri-driver` works
   unchanged. The gap is that "open the Tauri app and drive its embedded
   `WKWebView`" is not what `safaridriver` does — it opens Safari.
2. **`appium-mac2-driver`** — third-party, drives macOS apps via XCTest /
   accessibility APIs. Can click buttons in a Tauri window, but is *not* a
   WebDriver implementation for the embedded WebView; selectors target AX
   nodes, not the DOM. Useful for window-level automation, not for replacing
   the Linux/Windows WebDriver flow.
3. **WebKit's Remote Inspector / CDP-style protocol** — a `WKWebView` can be
   inspected by Safari Web Inspector, which implies a debug protocol exists,
   but it isn't a public W3C WebDriver endpoint. Driving it would mean
   re-implementing a meaningful slice of the WebDriver spec on top of the
   inspector protocol.

There is no shortcut that makes "Tauri WebDriver on macOS" work today the way
it works on Linux. That's the honest framing the rest of this doc proceeds
from.

## Proposed approach (incremental)

The scaffold (`src/macos.rs`) is structured so it can be filled in one layer
at a time without forcing a single big PR.

### Layer 1 — process supervision (this PR, partially done)

- Locate `safaridriver` (or a user-supplied `--native-driver`) on `$PATH`.
- Build the spawn `Command` with the same env vars and stdout policy used by
  the Linux/Windows path (`webdriver::native`).
- Probe the native driver port with a TCP connect + a raw `GET /status` so we
  can tell the user "the driver is up" vs "the driver crashed" without
  pulling in extra deps.

What's missing: actually launching the process from `main`. We don't do this
yet because spawning `safaridriver` succeeds (it's a real binary) but any
session created against it would target Safari, not the Tauri app — which
would silently mislead users. Failing fast with a pointer to this doc is the
honest behaviour for the scaffold.

### Layer 2 — capability translation (next)

Decide how `tauri:options` maps onto a macOS native shape. Two candidates:

- **`safari:options` (current `safaridriver` schema).** Easiest to implement,
  but only meaningful if we accept that the test target is Safari, not the
  Tauri binary. Probably not what users want.
- **`appium:options` against `appium-mac2-driver`.** Lets the test runner
  attach to the actual Tauri process by bundle ID. Loses DOM-level selectors
  but gains real-app automation. Likely the right default for end-to-end
  tests that just need to drive the app's UI.

The scaffold's `MacOsDriver::map_capabilities` is `unimplemented!()` so this
choice is explicit and reviewable in its own PR.

### Layer 3 — DOM-aware automation (open question)

The thing users actually want — DOM-level WebDriver against the embedded
`WKWebView` — almost certainly requires either:

- A `wry`-side hook that exposes a small WebDriver-shaped HTTP endpoint from
  inside the Tauri app, similar in spirit to what some third-party
  Tauri-Playwright bridges already do; or
- An out-of-process bridge that talks to the `WKWebView` Remote Inspector
  protocol and translates a useful subset of W3C WebDriver onto it.

Both are real engineering projects, and both arguably belong upstream of
`tauri-driver` (in `wry` or as a sibling crate). This document does **not**
commit `tauri-driver` to either path; it just acknowledges that Layer 2 alone
will not give Linux/Windows-equivalent behaviour and Layer 3 is where the
hard design work lives.

## What the scaffold ships

- `src/macos.rs` — `MacOsDriver` with binary resolution, command building,
  TCP-ready probe, raw `/status` probe, and explicit `unimplemented!()` for
  capability mapping and the server loop. Unit-tested without requiring
  `safaridriver` to be installed on the runner.
- `src/main.rs` — macOS branch parses CLI args (so `--help` works) and exits
  with a clear pointer to this document.
- `MACOS_DRIVER_DESIGN.md` — this file.
- README updated to remove the stale "Todo: Appium Mac2 Driver (probably)"
  bullet and replace it with a link here.

## Non-goals for the scaffold PR

- Working WebDriver sessions on macOS.
- Running existing Tauri WebDriver test suites against macOS.
- A decision on `safari:options` vs `appium:options` vs `wry`-side bridge.
- Any change to `crates/tauri-driver/Cargo.toml` dependencies. (The `which`
  crate already used on Linux/Windows covers binary lookup, so the scaffold
  intentionally avoids pulling in `serde_json`/`hyper` on the macOS path
  before we know which translation strategy we're committing to.)

## Related discussion / prior art

- Issue: <https://github.com/tauri-apps/tauri/issues/7068>
- Earlier root-cause comment on macOS support:
  <https://github.com/tauri-apps/tauri/issues/5551#issuecomment-1304348684>
- Apple's Safari WebDriver setup:
  <https://developer.apple.com/documentation/safari-developer-tools/macos-enabling-webdriver>
- `appium-mac2-driver`: <https://github.com/appium/appium-mac2-driver>
