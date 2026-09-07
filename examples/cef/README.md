# CEF example

A full Tauri application running on the Chromium Embedded Framework, showcasing
the APIs [`tauri-runtime-cef`](../../crates/tauri-runtime-cef) adds on top of the
portable Tauri ones. Every panel of the app exercises one of them, and the Rust
side of each is commented in [`src-tauri/src`](./src-tauri/src).

The runtime is selected by depending on `tauri-runtime-cef` and passing
`tauri_runtime_cef::Cef` to `tauri::Builder::runtime`. That dependency is also
how `tauri-build` and the Tauri CLI detect a CEF app, which is what makes them
ship the CEF binary distribution, sign it with the entitlements Chromium's JIT
needs, and run the app from inside an `.app` bundle in `tauri dev` on macOS —
CEF launches its helper apps by path from inside the bundle, so it cannot run as
a bare executable there.

## Running the example

Compile the CLI once, from the root of the repository:

```bash
pnpm i
pnpm build:debug
```

Then, from this directory:

```bash
pnpm tauri dev
```

The first build downloads the CEF binary distribution (about 1 GB) into
`{user cache}/tauri-cef`, or into `$CEF_PATH` when that is set. To build and run
the bundled application instead:

```bash
pnpm tauri build
```

## What it shows

### Selecting and configuring the runtime — [`runtime_config.rs`](./src-tauri/src/runtime_config.rs)

Everything `Cef` configures before the application is built. Each one is driven
by an environment variable so its effect can be seen without editing the file:

| Variable                       | Values                                           | What it changes                                                                                                      |
| ------------------------------ | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| `CEF_EXAMPLE_SANDBOX`          | `auto` (default), `required`, `disabled`         | Chromium's process sandbox. `auto` drops it only for an AppImage on a system that cannot sandbox at all.             |
| `CEF_EXAMPLE_SECRET_STORAGE`   | `auto` (default), `mock`, `system`               | Which key the cookie jar is encrypted with. Switching makes cookies stored under the previous key unreadable.        |
| `CEF_EXAMPLE_SAFE_BROWSING`    | `on` (default), `off`                            | The `safebrowsing.enabled` profile preference, which the runtime leaves on.                                          |
| `CEF_EXAMPLE_PASSWORD_MANAGER` | `off` (default), `on`                            | The `credentials_enable_service` preference, which the runtime turns off so the "Save password?" bubble stays away.  |
| `CEF_EXAMPLE_LOG_SEVERITY`     | `verbose`, `info`, `warning`, `error`, `disable` | What CEF writes to the log file. Unset keeps the runtime's default: INFO in development, WARNING in release.         |
| `CEF_EXAMPLE_CHROMIUM_ARGS`    | `off` (default), `on`                            | Whether a release build honours Chromium switches on its own command line. Development builds always honour them.    |
| `CEF_EXAMPLE_AUTOPLAY`         | `on` (default), `off`                            | Whether `--autoplay-policy=no-user-gesture-required` is set, which the page reads back as an `AudioContext`'s state. |

The app also sets raw Chromium switches — one through `command_line_arg` and two
through `command_line_args` — an `Accept-Language` list, a cache directory and a
log file of its own, names the `cef_api_version` both of its processes declare,
and reaches `cef::Settings` directly through `Cef::with_settings`. Three of those
are read back from the page — `navigator.languages`, the state of a fresh
`AudioContext` and the `window.gc` that `--js-flags=--expose-gc` adds — so you
can tell the setting reached Chromium, and in the last case the _renderer_, and
not just the Rust struct. Chromium exposes no API for the autoplay policy, so
that row is the behaviour itself: a context created with no user gesture behind
it starts `running` only because `--autoplay-policy=no-user-gesture-required` is
set.

`Cef::locale` is the one method left unset on purpose, and the configuration
table says why: the bundler packages only the `en-US` locale pak, so naming
another locale leaves Chromium unable to load the strings of its own UI.

### Per-webview APIs — [`main.rs`](./src-tauri/src/main.rs) and [`commands.rs`](./src-tauri/src/commands.rs)

- **`on_console_message`** — the renderer's console output, in Rust, with no
  DevTools open.
- **`on_frame_event`** — the native lifecycle of every frame of the browser,
  child frames included, which Tauri's portable navigation callbacks do not
  report.
- **`allow_chrome_commands`** — a Chrome style browser keeps its whole
  accelerator table live even hosted as a child view with no browser UI, so the
  runtime swallows the families that make no sense in an app window. The
  accelerators panel opens a window keeping the families you tick, so Ctrl+P,
  Ctrl+H and Alt+Left can be compared with and without.
- **`browser_runtime_style`** — Chrome or Alloy style, per browser rather than
  per application.
- **`WebviewBuilderCefExt`** — the same four builder methods on
  `tauri::webview::WebviewBuilder`, for a window hosting several webviews. It
  follows Tauri's multiwebview API behind the `unstable` feature, which is why
  the example enables that feature on `tauri-runtime-cef`. The child webviews
  panel opens one window with a Chrome style browser and an Alloy style browser
  side by side.
- **`send_dev_tools_message` / `on_dev_tools_protocol`** — the Chrome DevTools
  Protocol, with request ids from `allocate_devtools_message_id()`. Every
  observer on a browser sees every result, so a hardcoded id can consume somebody
  else's answer.
- **`with_cef_webview`** — the native CEF handle and the state CEF sampled for
  that callback: browser identity, document admission, native window identity,
  visibility, bounds, observed JavaScript dialogs, and the CEF-owned popups of
  this browser. The panel samples any webview by label — a child webview of a
  multiwebview window included — so a webview can be inspected from a window
  that is not it. It also shows `Webview::for_document` selecting a browser back
  out of the family by `NativeDocumentToken`, and reads the raw `cef::Browser`
  that `Webview::browser()` hands over for the main frame URL, the frame count,
  the zoom level and whether DevTools is open.
- **`AsCefWindowOpener`** — CEF reports the opener's main-frame URL straight to
  the new-window callback, so it can be read without a blocking webview getter,
  which on this runtime can deadlock the UI thread. The popups panel opens both a
  Tauri window (`NewWindowResponse::Create`) and a CEF-owned browser
  (`NewWindowResponse::Allow`) so the difference is visible.
- **`on_permission_request`** — a portable Tauri API with three CEF specifics
  worth knowing, all commented on `decide_permission` in `main.rs`: decisions
  persist per origin, permissions Tauri has no kind for arrive as
  `PermissionKind::Other`, and `DisplayCapture` is never granted by an `Allow`.
- **`deep_link_schemes`** — a deep link delivered to an already running app goes
  through Chromium's process singleton, which relays a command line CEF has by
  then cleared; the runtime restores the URL onto it. Try
  `open tauri-cef-example://hello` on macOS, or run the built executable again
  with `tauri-cef-example://hello` as its argument elsewhere. Schemes declared
  under `plugins > deep-link > desktop` in the config are picked up automatically,
  so an app using the deep link plugin does not name them twice.

### The entry point

`#[tauri_runtime_cef::cef_entry_point]` on `main` handles the other side of a CEF
application: the same executable is also its own renderer, GPU, network and
utility process, and the attribute runs the helper side and returns before the
Tauri application is ever built. It expands to a `--type=` check around
`run_cef_helper_process()`, which is public for an application whose entry point
cannot take an attribute.

`prepare_macos_application()` is the other half, and the runtime already calls it
when it initializes. It exists for a macOS application that shows native UI of
its own — a recovery dialog, say — _before_ the runtime starts: AppKit's
application singleton has to be the CEF-compatible one, and whoever creates
`NSApplication` first decides which class that is.

### Not shown

`AsCefWebviewDispatcher` and `AsCefWebviewAttributes` are the traits the
extension methods above are written against — they are what lets one call work
on both `CefRuntime` and the type-erased `tauri::DynRuntime` — and an
application only names them itself when it writes an extension of its own.
