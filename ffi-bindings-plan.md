# Plan: FFI Binding Layer for the `tauri` Crate

Goal: let non-Rust hosts (C first, then Node.js, Python, Deno, …) build and drive a
Tauri desktop app in-process. One hand-designed C ABI is the single product; every
language binding is a thin, eventually *generated* shim over that ABI — no pyo3, no
napi-rs, no per-language native code.

Scope for v0: **desktop only** (Wry runtime). Mobile stays with the existing Swift/Kotlin
bridges.

---

## 1. Ground truth (verified against this repo)

These facts shape the whole design. File references are current as of `feat/bindings`.

| Fact | Evidence | Consequence |
|---|---|---|
| A `Context` can be built 100% at runtime: `Context::new` is `pub` (`#[doc(hidden)]`, "unstable") | `crates/tauri/src/lib.rs:484` | No `generate_context!`/tauri-build needed by FFI consumers. In-tree crate absorbs the instability. |
| `Config` derives `Deserialize` (+ `deny_unknown_fields`); `parse_json` is public | `crates/tauri-utils/src/config.rs:3637`, `config/parse.rs:340` | Hosts pass `tauri.conf.json`-shaped JSON strings at runtime. |
| `Assets` is an object-safe trait (`Box<dyn Assets<R>>`); `NoopAsset` is the minimal template | `crates/tauri/src/lib.rs:312`, `src/test/mod.rs:81` | Serve app frontends from a directory, a dev-server URL, or host callbacks — no embedding step. |
| ACL is runtime-resolvable: `Resolved::resolve` is pub; `dynamic-acl` (default feature) enables `Manager::add_capability` with JSON/TOML strings | `crates/tauri-utils/src/acl/resolved.rs:85`, `crates/tauri/src/ipc/authority.rs:150`, `src/lib.rs:813` | Capabilities become runtime JSON inputs. Plugin permission *manifests* are known at cdylib build time (plugins are compiled in), so we embed them then. |
| `App::run_iteration` is **deprecated** (busy-loops; tao 0.35 has `run_return` but no `pump_events`) | `crates/tauri/src/app.rs:1467`, `crates/tauri-runtime-wry/src/lib.rs:3094-3112` | A "host pumps the loop" model is not viable today. Foundation = blocking run. |
| `App::run_return` blocks, returns the exit code, and gives the main thread back | `crates/tauri/src/app.rs:1405`, CHANGELOG (PR #12668) | The FFI `run` wraps `run_return`; hosts regain control after exit for cleanup. |
| macOS: event loop must be created *and* run on the OS main thread (`any_thread` not exposed there) | `crates/tauri-runtime/src/lib.rs:412`, `app.rs:1634` | `tauri_app_new` + `tauri_app_run` are main-thread-only. Everything else is callable from any thread. |
| `AppHandle` is `Send + Sync + Clone` | `crates/tauri/src/app.rs:386,476`; asserts at `webview/mod.rs:2372` | Handle-based C API is safe to call from any host thread while the loop runs. |
| `Builder::invoke_handler` takes one plain closure `Fn(Invoke<R>) -> bool`; `InvokeResolver` is `Send`, `Clone`, resolvable later from any thread | `app.rs:1658`, `ipc/mod.rs:286-421` | Commands need no Rust macros: dispatch by name to the host, respond asynchronously via a resolver handle. |
| Rust event listeners run inline on the emitting thread; `emit_str`/`emit_str_to` accept pre-serialized JSON | `src/event/listener.rs:190-204`, `lib.rs:954,993` | Cheap event bridge; JSON-in/JSON-out at the boundary. |
| Windows are creatable after launch, from any thread, from a `WindowConfig`: `WebviewWindowBuilder::from_config` | `webview/webview_window.rs:150` (Windows sync-handler deadlock caveat at :115) | Window creation = one FFI call with a JSON config. Docs must steer hosts off "create window synchronously inside an invoke callback" on Windows. |
| Veto-style APIs (`prevent_close`, `prevent_exit`) are synchronous within the callback | `RunEvent` at `app.rs:220` | Direct C callbacks can veto in-place; queue-based hosts (Node) get a pre-set *policy* API instead (§4.4). |
| No existing desktop C ABI in-tree (only mobile `start_app`/plugin bridges) | `tauri-macros/src/mobile.rs:87`, `tauri/src/ios.rs` | Greenfield — we set the conventions. |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Language packages (thin, no native code, generated in M3)  │
│  bindings/node (koffi)  bindings/deno (Deno FFI)            │
│  bindings/python (cffi ABI mode)  bindings/c (header+docs)  │
└──────────────────────────┬──────────────────────────────────┘
                           │  stable C ABI (cdylib/staticlib)
┌──────────────────────────┴──────────────────────────────────┐
│  crates/tauri-ffi                                            │
│  • extern "C" layer: naming/ownership/error conventions,     │
│    catch_unwind at every entry point                         │
│  • safe facade: monomorphized to Wry, handle registry        │
│    (u64 ids → AppHandle/WebviewWindow/Resolver/Channel…),    │
│    event queue + callback dispatch, JSON (de)serialization   │
│  • runtime Context assembly: Config JSON, Assets impls       │
│    (dir / callbacks / dev URL), embedded ACL manifests +     │
│    runtime capabilities                                      │
└──────────────────────────┬──────────────────────────────────┘
                           │  normal Rust dependency
┌──────────────────────────┴──────────────────────────────────┐
│  tauri (+ tauri-runtime-wry, tauri-utils, opt-in plugins)   │
└─────────────────────────────────────────────────────────────┘
```

Design principles:

1. **One binding, N consumers.** All language packages talk to the same cdylib through
   dynamic FFI (koffi / `Deno.dlopen` / cffi ABI mode). No per-language compiled glue →
   packages are pure script + a prebuilt binary, and a new language is "write (later:
   generate) one file".
2. **Handles, not pointers.** Objects are opaque `uint64_t` ids into a generation-checked
   registry (slotmap). A stale id returns an error instead of UB — critical when callers
   are GC'd languages.
3. **JSON at the boundary for structured data.** Matches tauri's serde-everywhere design
   (`WindowConfig`, capabilities, events, invoke payloads are all serde types already).
   Scalars/strings stay native C types. Structs can be promoted to real C layouts later
   per hot path (none of these calls are hot — the webview↔frontend path never crosses
   this boundary).
4. **Two callback delivery modes, one core** (§4): direct C function pointers for hosts
   that tolerate foreign-thread reentry (C, Python), and a poll/queue mode for hosts that
   don't (Node, Deno).
5. **Single source of truth for the API surface** (§6): every exported function is
   described in a machine-readable manifest; generators emit the header and the language
   shims from it. Adding one function to the facade = every language gets it.

### Why not the alternatives

- **pyo3 + napi-rs per language** — best per-language DX, but N parallel native
  codebases tracking the same facade, N build pipelines, N sets of threading bugs.
  Explicitly rejected per project goal (reuse the C layer).
- **Generate from rustdoc JSON** — still nightly-only/unstable as of mid-2026 with no
  stabilization on the project-goals list, and it captures none of the semantics FFI
  needs (ownership transfer, callback thread contracts, veto semantics, handle
  lifetimes). The tauri API is also too Rust-idiomatic (generics over `R: Runtime`,
  closures, builders) to map mechanically — a hand-designed facade is needed
  regardless, so annotate the facade, don't scrape docs.
- **UniFFI** — produces Python/Kotlin/Swift first-party, and since 2026 there is a
  koffi-based Node generator (`uniffi-bindgen-node-js`, v0.0.x, single-author) plus
  livekit's explicitly-experimental `uniffi-bindgen-node`. Still disqualified as the
  backbone: it produces **no C header as a product** (its scaffolding ABI is internal
  and unstable across uniffi versions) and C is a first-class target here. Worth
  revisiting as an *additional* layer if we later want idiomatic Swift/Kotlin desktop
  bindings.
- **interoptopus** — closest prior art (inventory → backends), but as of 2026 only the
  C# backend is Tier 1; the **C and Python backends are suspended** ("contributors
  wanted"). Betting a C-first project on its two suspended backends is not defensible.
- **Sidecar process + JSON-RPC** (neutralino-style) — sidesteps the main-thread problem
  entirely but adds process management, IPC latency, packaging complexity, and kills
  in-process embedding (shared memory, same-process state). Could be layered *on top of*
  this ABI later (the manifest would generate the RPC schema too).

---

## 3. Threading model (the crux)

Tauri must own the **OS main thread** for the app's lifetime (macOS hard requirement;
uniform contract on all platforms for v0). The FFI contract:

- **Main-thread-only:** `tauri_app_new`, `tauri_app_run`. `tauri_app_run` wraps
  `App::run_return`, blocks until exit, returns the exit code, and runs
  `cleanup_before_exit` semantics. Documented: "call from the process main thread".
- **Any-thread:** everything else (window ops, emit, listen, resolver resolve/reject,
  channel send, capability add). Backed by `AppHandle` clones + dispatchers, which queue
  through the running loop.

How each host lives with a blocked main thread:

- **C**: natural — same as running any GUI toolkit. Callbacks arrive on the main thread
  (run events) or the emitting thread (events); documented per function in the manifest.
- **Python**: natural for scripts (`app.run()` blocks like Qt/Tk). cffi callbacks
  re-acquire the GIL from any thread, so direct-callback mode works while the main
  thread is parked inside C. The package marshals handler calls onto a dispatch thread
  or the user's asyncio loop (`call_soon_threadsafe`) — package-level sugar, not ABI.
- **Node.js**: the main JS thread *is* the OS main thread, and blocking it kills libuv —
  so the package inverts the layout: `launch()` re-runs user app code in a
  `worker_threads.Worker` (own event loop), then parks the main thread in
  `tauri_app_run` via a plain blocking koffi call. The worker drives everything through
  any-thread FFI calls and consumes events by looping on `tauri_events_next` using
  koffi's async call mode (executes on the libuv threadpool, resolves a promise).
  No native callbacks into JS at all → no ThreadsafeFunction machinery, pure C ABI reuse.
- **Deno**: same worker-bootstrap shape via `Deno.dlopen` + `nonblocking: true` FFI calls
  for the event loop poll. (On Windows/Linux, `Builder::any_thread` could later allow a
  no-worker mode; macOS never can.)

**Roadmap bet (tracked, not load-bearing):** contribute a real `pump_events` to tao
(winit parity). That would enable a cooperative single-thread embedding mode
(`tauri_app_pump(timeout_ms)`) so Node/Deno could skip the worker bootstrap. The ABI
reserves room for it; nothing in v0 depends on it. Today's `run_iteration` is deprecated
precisely because it busy-loops (`tauri-runtime-wry/src/lib.rs:3108-3112`) — do not
build on it.

---

## 4. C ABI design

### 4.1 Conventions

- Names: `tauri_<module>_<verb>` (`tauri_app_run`, `tauri_window_set_title`,
  `tauri_invoke_resolve`). Prefix guarantees no symbol clashes.
- Every fallible function returns `int32_t` (`TAURI_OK = 0`, negative = error class);
  out-values via pointer params. Per-thread `tauri_last_error_message()` returns a
  borrowed UTF-8 string with details.
- Strings: UTF-8, NUL-terminated. Returned strings are owned by the caller and freed
  with `tauri_string_free`. Input strings are borrowed for the duration of the call.
- Handles: `uint64_t` (0 = invalid). Freed with `tauri_handle_close` (drops the registry
  entry; underlying object follows normal Arc semantics).
- Callbacks: C fn pointer + `void* user_data`. Manifest records each callback's thread
  contract. All extern entry points wrap bodies in `catch_unwind` → panic becomes
  `TAURI_ERR_PANIC` + message, never unwinds across the boundary.
- ABI versioning: `tauri_ffi_abi_version() -> uint32_t` (bumped on breakage) +
  `tauri_ffi_version()` (semver string). Additive growth preferred; the manifest diff is
  the review artifact for ABI changes.

### 4.2 App construction & lifecycle

```c
uint64_t tauri_app_builder_new(const char* config_json);        // tauri.conf.json shape
int32_t  tauri_app_builder_set_assets_dir(uint64_t b, const char* path);
int32_t  tauri_app_builder_set_assets_callback(uint64_t b, TauriAssetGetFn get, void* user_data);
// dev-server URL comes via config (build.devUrl / window url), no special API
int32_t  tauri_app_builder_set_icon_rgba(uint64_t b, const uint8_t* rgba, uint32_t w, uint32_t h);
int32_t  tauri_app_builder_add_capability(uint64_t b, const char* capability_json);
int32_t  tauri_app_builder_set_assets_archive(uint64_t b, const char* path); // packed by `tauri build`
int32_t  tauri_app_builder_set_dev(uint64_t b, bool dev); // WebviewUrl::App -> devUrl (CLI dev mode)
// host-defined plugins (implemented; see 4.5): create, configure, attach
uint64_t tauri_plugin_new(const char* name, uint64_t* out_plugin);
int32_t  tauri_plugin_set_init_script(uint64_t plugin, const char* js);
int32_t  tauri_plugin_register_command(uint64_t plugin, const char* name);
int32_t  tauri_app_builder_add_plugin(uint64_t b, uint64_t plugin); // consumes the plugin handle
int32_t  tauri_app_build(uint64_t b, uint64_t* out_app);        // main thread; creates event loop
int32_t  tauri_app_run(uint64_t app, TauriRunEventFn cb /*nullable*/, void* user_data,
                       int32_t* out_exit_code);                 // main thread; blocks
int32_t  tauri_app_exit(uint64_t app, int32_t code);            // any thread
```

Internally: parse `Config` (`parse_json`), build `Assets` impl (dir walker / callback
adapter / `NoopAsset` when everything is remote-URL), assemble `Context::new(...)` with
`PackageInfo` from config, `Pattern::Brownfield`, and a `RuntimeAuthority` built from
**embedded manifests + runtime capabilities** (§4.5). Windows declared in
`config.app.windows` are created by tauri as usual at startup.

### 4.3 Commands (invoke) — the surface app developers live on

```c
int32_t tauri_commands_register(uint64_t app, const char* name);     // handler returns true only for these
int32_t tauri_commands_set_handler(uint64_t app, TauriInvokeFn cb, void* user_data); // direct mode
// TauriInvokeFn(user_data, const TauriInvokeMsg* msg, uint64_t resolver)
//   msg: command, window label, JSON payload (or raw bytes ptr+len), headers JSON
int32_t tauri_invoke_resolve(uint64_t resolver, const char* json);   // any thread, any time
int32_t tauri_invoke_reject(uint64_t resolver, const char* json);
int32_t tauri_channel_send(uint64_t channel, const char* json);      // streaming responses
```

Backed by one `Builder::invoke_handler` closure: known command → package into registry +
deliver (direct callback or event queue), return `true`; unknown → `false` (tauri emits
"command not found"). Resolver handles wrap the `Send + Clone` `InvokeResolver` —
resolving from any host thread later is supported by tauri today (`ipc/mod.rs:324-421`).

### 4.4 Events & run-events — two delivery modes

```c
// Direct mode (C, Python): fires on the emitting thread / main thread
int32_t tauri_events_listen(uint64_t app, const char* event, const char* target_json /*nullable*/,
                            TauriEventFn cb, void* user_data, uint32_t* out_listener_id);
int32_t tauri_events_unlisten(uint64_t app, uint32_t listener_id);
int32_t tauri_events_emit(uint64_t app, const char* event, const char* payload_json);
int32_t tauri_events_emit_to(uint64_t app, const char* target_json, const char* event, const char* payload_json);

// Queue mode (Node, Deno; also usable from C): one serialized stream of everything —
// run events, subscribed app events, invokes (when no direct handler is set)
int32_t tauri_events_next(uint64_t app, uint32_t timeout_ms, char** out_event_json);
// returns TAURI_OK (event), TAURI_ERR_TIMEOUT, or TAURI_ERR_CLOSED (app exited)
```

Veto semantics: direct-mode `TauriRunEventFn` gets an out-param struct
(`bool* prevent_exit` / `prevent_close`) usable synchronously. Queue mode can't veto
after the fact, so policies are pre-set:

```c
int32_t tauri_window_set_close_policy(uint64_t win, TAURI_CLOSE_ALLOW | TAURI_CLOSE_PREVENT_AND_NOTIFY);
int32_t tauri_app_set_exit_policy(uint64_t app, ...);
```

`PREVENT_AND_NOTIFY` = core prevents the close, queues the event; host decides and calls
`tauri_window_destroy` itself. This is the standard pattern in embedder APIs and needs
no synchronous reentry.

### 4.5 ACL / capabilities

- Plugins usable through this ABI are **compiled into the cdylib** behind cargo features
  (`ffi-tray`, `ffi-dialog`, …), so their permission manifests are known at *cdylib*
  build time. `tauri-ffi/build.rs` embeds the collected `acl-manifests.json` (same
  data tauri-build assembles from plugin build-script metadata — exact reuse surface to
  confirm in M0) as a static JSON blob.
- At runtime: host-supplied capability JSON strings (plus a default "main window gets
  `core:default`" capability unless disabled) are resolved via `Resolved::resolve` →
  `RuntimeAuthority::new`; post-launch additions use `Manager::add_capability`
  (`dynamic-acl` is a default feature).
- Host-registered commands ride on `has_app_acl = false` (app commands ungated), same as
  a macro-built app that defines no app permissions.
- **Host-defined plugins (implemented 2026-07-09).** A host builds a plugin from its own
  language — a name, an optional JS init script injected into every webview, and a set of
  commands invoked from the frontend as `plugin:<name>|<command>`. Unlike app commands,
  plugin commands are *always* ACL-gated (tauri rejects `plugin:*` invokes whose access
  doesn't resolve). Since host plugins have no build-time manifest, `tauri-ffi` synthesizes
  one at runtime: a `Manifest` with an `allow-all` permission (`commands.allow` = every
  registered command) under a `default` set, inserted into the acl map keyed by plugin
  name *before* `RuntimeAuthority::new`; the default capability is then extended with
  `<name>:default` for windows `["*"]`. Each plugin attaches a `tauri::plugin::Builder`
  (name `Box::leak`ed to `&'static str`) whose `invoke_handler` funnels into the same event
  queue with an added `"plugin"` field; language shims route handlers by the composite
  `plugin:<name>|<command>` key. `core`/`tauri` are reserved names (surfaced as a build
  error). Lifecycle hooks (`on_page_load`, `on_navigation`, …) deliberately deferred.

### 4.6 Windows & webviews (initial op set)

`tauri_window_create(app, window_config_json, *out_win)` →
`WebviewWindowBuilder::from_config`. Ops: label/list/get, show/hide/close/destroy,
title, inner/outer size + position, fullscreen/maximize/minimize, focus, decorations,
always-on-top, `eval(js)`, `navigate(url)`, `reload`, `open_devtools`, `set_zoom`.
Each is a one-line dispatch through the handle — mechanically addable, which is exactly
what the manifest/codegen path (§6) makes cheap.

---

## 5. Language packages

All are **pure script + prebuilt cdylib**; hand-written in M2 to learn the idioms, then
regenerated from the manifest in M3 (generated low-level layer + hand-written idiomatic
sugar on top — sugar is per-language and stays manual by design).

| | loader | events | packaging |
|---|---|---|---|
| C | `tauri_ffi.h` (cbindgen → generated) | direct callbacks or queue | release tarball: header + dylib/static lib + `.pc` file |
| Node | koffi (no node-gyp, prebuilt-free) | worker bootstrap (§3) + async `tauri_events_next` | npm `@tauri-apps/node`, per-platform binary via optionalDependencies (esbuild pattern) |
| Deno | `Deno.dlopen` (`--allow-ffi`) | worker bootstrap + `nonblocking` FFI | JSR package; dylib fetched/cached per platform |
| Python | cffi ABI mode (no compiler at install) | direct callbacks; sugar marshals to dispatch thread / asyncio | wheels embedding the dylib per platform |

Node sketch (what M2 must make feel natural):

```js
// main.js — process entry, parks the OS main thread
import { launch } from "@tauri-apps/node";
launch(new URL("./app.js", import.meta.url), { config });

// app.js — runs in a worker with a live event loop
import { app } from "@tauri-apps/node/worker";
app.command("greet", async ({ name }) => `Hello ${name}!`);
app.on("ready", async () => {
  const win = await app.createWindow({ label: "main", url: "index.html" });
});
```

Python sketch:

```python
from tauri import App

app = App(config)

@app.command("greet")
def greet(payload):
    return {"message": f"Hello {payload['name']}!"}

app.run()  # blocks the main thread, like every Python GUI toolkit
```

---

## 6. Single source of truth: manifest + codegen

**Decision (2026-07-09): in-house, manifest-first.** A hand-authored
`crates/tauri-ffi/api-manifest.json` describes the ABI (functions, param/result kinds,
error codes, docs, thread contract, blocking flags); a zero-dependency generator
(`bindings/bindgen/generate.mjs`) renders every consumer artifact and cross-checks the
manifest against the `extern "C"` fns actually implemented in `lib.rs` (drift = hard
error). Implemented and validated — the fixture runs on fully generated declarations.

Generated today from one manifest entry per function:

| artifact | consumer |
|---|---|
| `bindings/c/tauri_ffi.h` | C (docs + thread contracts in comments) |
| `bindings/node/src/ffi-decls.js` | Node (koffi declarations + error codes) |
| `bindings/deno/symbols.ts` | Deno (`dlopen` map; `blocking` → `nonblocking: true`) |
| `bindings/python/tauri_ffi_cdef.py` | Python (cffi ABI-mode `cdef` + error codes) |

`--check` mode makes CI fail on stale outputs; the manifest diff is the ABI-review
artifact on every PR.

Why in-house won the evaluation (full comparison in the alternatives list, §2):

- The ABI is deliberately tiny and uniform — 9 param kinds, 4 result kinds, no
  callbacks (the queue model eliminated them). The entire generator is ~250 lines of
  template literals; any framework's integration cost exceeds that permanently.
- The frameworks fail the requirements table on facts, not taste: interoptopus's C and
  Python backends are suspended; UniFFI emits no C header and its Node generator is
  v0.0.x; rustdoc JSON is still unstable and semantics-free; cbindgen covers only the
  header and reads none of our semantic metadata (ownership, blocking, thread).
- The manifest carries exactly the semantics that make dynamic-FFI hosts correct:
  `blocking` becomes Deno's `nonblocking`/koffi's async guidance, `out_owned_str`
  encodes who frees, `thread: main` lands in the header docs.

Later phases (unchanged in spirit):

- **M3a — generate the Rust extern shims too.** `lib.rs` splits into hand-written safe
  impl fns + a generated `extern "C"` layer (cstr conversion, out-params,
  `catch_unwind`) rendered from the same manifest. Drift then becomes a compile error
  instead of a lint. The syn/proc-macro (`#[tauri_ffi::export]` + inventory) route
  stays documented as the fallback if manifest authoring ever chafes — the generator
  and manifest format survive that migration unchanged.
- **M3b** — TS high-level layer shared by Node/Deno, generated docs pages, manifest
  schema validation.

---

## 7. Repo layout & workspace changes

```
crates/tauri-ffi/            # facade + extern "C" layer (crate-type: cdylib, staticlib, rlib)
crates/tauri-ffi/macros/     # M3: #[tauri_ffi::export] proc macro
crates/tauri-ffi/bindgen/    # M3: manifest dump + generators
bindings/c/                  # header (generated), examples, pkg-config template
bindings/node/               # npm package (+ examples)
bindings/deno/
bindings/python/
```

Keeping `tauri-ffi` **in this workspace** is load-bearing: it version-locks against the
unstable `Context::new` / `RuntimeAuthority::new` surface and lets us stabilize or adjust
those constructors in the same PR when needed (they're ours). JS packages could
alternatively live under `packages/` to match `packages/cli` — decide at M2; grouping all
consumers under `bindings/` is the default proposal.

Features on `tauri-ffi`: `default = ["queue", "direct-callbacks"]`, plus `ffi-tray`,
`ffi-menu`, `ffi-dialog`, … pulling the corresponding plugin crates + their manifests.

---

## 8. Milestones

**M0 — Spike (risk burn-down, ~days).** Throwaway-quality but honest: hand-rolled cdylib
with `app_new(config_json)` → `window_create` → `run_return`; a 50-line C demo that opens
a window loading a URL, JS `listen`/`emit` round-trip through a runtime-resolved
`core:default` capability. Validates on macOS + Windows + Linux: runtime Context recipe,
ACL manifest embedding (the one "confirm exact mechanism" item, §4.5), and the blocking
run contract. Exit criteria: demo runs on all three platforms; unknowns list is empty or
has owners.

> **Status (2026-07-08):** implemented and validated on macOS — `crates/tauri-ffi`
> (~20 fns), `bindings/c/tauri_ffi.h` + C example, `bindings/node` (koffi + worker
> bootstrap) with a self-validating fixture at `bindings/node/examples/hello`
> (invoke round-trip, host↔frontend events, `withGlobalTauri` injection, runtime
> `core:default` capability). ACL embedding confirmed: `tauri-ffi/build.rs` collects
> `DEP_TAURI_*` manifests via `tauri_utils::acl::build::read_permissions()`.
> Remaining M0: Windows + Linux runs.
>
> **Update (2026-07-09):** window surface landed via the manifest pipeline — ABI v2
> adds `tauri_window_create` (WindowConfig JSON), `tauri_app_get_window` /
> `tauri_app_window_labels`, and ~35 `tauri_window_*` getter/setter/action fns
> mirroring `tauri::WebviewWindow` on window handles (label/title/url, size &
> position in physical or logical pixels, visibility/focus/fullscreen state,
> show/hide/center/maximize/minimize/close/destroy, eval/navigate/reload/zoom).
> `tauri_webview_eval(app, label, js)` was removed in favor of methods on window
> handles. Node exposes a `WebviewWindow` class + `app.createWindow()` /
> `app.getWindow()`; fixture validates creation, getters and destroy end-to-end.

**M1 — C API v0.1.** Full §4 surface (~60 fns), conventions doc, cbindgen header, error
model, both callback modes, invoke round-trip with resolver + channel, close/exit
policies, C examples (hello-window, commands, events). CI: build matrix + smoke tests
(Linux under xvfb with webkit2gtk-4.1; macOS/Windows runners headed).

**M2 — Node + Python, hand-written.** Node package with worker bootstrap + koffi +
async event pump; Python package with cffi + blocking run + callback dispatch. Both ship
the same demo app as C. This is where boundary idioms get discovered — feed every
irritation back into ABI tweaks *before* codegen freezes patterns.

> **Status (2026-07-09):** done, plus Deno ahead of schedule — all three language
> packages run on their generated artifacts and pass the same smoke trace (invoke
> round-trip, host↔frontend events, window create/getters/destroy) on macOS:
> `bindings/node` (koffi + worker_threads), `bindings/python` (cffi ABI mode;
> blocking `App.run()` on the main thread + daemon pump thread — no worker
> inversion needed), `bindings/deno` (`Deno.dlopen`; sync `tauri_app_run` on the
> main thread, `nonblocking` queue poll, app handle passed to the Worker via URL
> query params since Deno lacks `workerData`). The Deno emitter learned one rule:
> `blocking && thread == main` must NOT be `nonblocking`. The ABI version constant
> is now generated into the Rust crate from the manifest by build.rs — the
> per-language `tauri_ffi_abi_version()` startup check caught it drifting.

> **Update (2026-07-09): host-defined plugin system landed (ABI v3).** Four new ABI
> fns (`tauri_plugin_new` / `tauri_plugin_set_init_script` /
> `tauri_plugin_register_command` / `tauri_app_builder_add_plugin`) let a host define a
> plugin from its own language: a name, an optional JS init script (injected into every
> webview before page scripts run), and commands invoked as `plugin:<name>|<command>`.
> Rust side (`crates/tauri-ffi/src/plugins.rs` + `build_app`): a `tauri::plugin::Builder`
> per plugin whose `invoke_handler` funnels into the shared event queue with a `"plugin"`
> field, plus a runtime-synthesized ACL `Manifest` (`allow-all` permission) so the
> always-gated plugin invokes resolve — see §4.5. The minimal C example stays plugin-free.
>
> **Higher-level wrapper (same day).** Each scripting language ships a dependency-free
> `definePlugin(name)` / `Plugin(name)` builder (`@tauri-apps/node/plugin`,
> `@tauri-apps/deno/plugin`, `tauri_ffi.Plugin`) that bundles the name, init script and
> `command(name, handler)` handlers into one packageable object — mirroring
> `tauri::plugin::Builder`, so a plugin can live in its own npm/JSR/PyPI package. Apps
> pass the object to `launch({ plugins })` (metadata + ACL, main thread) and `app.plugin(p)`
> (handlers, worker) — Python's single call does both. The `plugin:<name>|<command>` wire
> format is fully internal: the worker keys handlers by it, and the plugin's own init
> script installs a frontend guest API (`window.demo.echo(...)`) so app + frontend code
> never type it. The `demo` plugin (init script installing `window.demo` + `echo` command)
> lives in a separate `demo-plugin` module in all three `hello` fixtures and is exercised
> end-to-end — the frontend reads `window.__DEMO_PLUGIN__` (proves injection) and calls
> `window.demo.echo()` (proves ACL resolution through the guest API).

> **Update (2026-07-09): Tauri CLI `dev`/`build` for bindings projects (ABI v4).**
> The CLI's project interface is now a real trait (`interface::Interface` +
> `AppSettings::out_dir`, `crates/tauri-cli/src/interface/mod.rs`): `dev.rs`/`build.rs`
> are generic over it, `run_hook`/`bundle()` take any `Interface`, and the only leaked
> Rust-specific (the cargo target-dir check in `build::setup`) is gated on project kind.
> Dispatch: `Cargo.toml` present → `Rust` (unchanged, now a trait impl by delegation);
> otherwise → new `interface/bindings.rs`, which is language-agnostic — it spawns the
> `build > runner` command (e.g. `{"cmd":"node","args":["main.js"]}`) with the fully
> merged config in `TAURI_CONFIG` + `TAURI_DEV=true`, restarts on file changes
> (`.taurignore` + builtin ignores + frontendDist excluded), and on `build` packs
> `frontendDist` into `target/{debug,release}/<productName>.assets` (magic `TAURIPK1`,
> JSON index + blob). FFI: `tauri_app_builder_set_assets_archive` (`ArchiveAssets`) and
> `tauri_app_builder_set_dev` — the dev flag rewrites `frontend_dist = Url(devUrl)`,
> which reproduces a Rust dev build exactly because the runtime resolves
> `WebviewUrl::App` *and* the Local ACL origin via `get_app_url()` (manager/mod.rs)
> from that value; no dev-flavored cdylib needed. Bindings runtime gained
> tauri.conf.json auto-discovery (next to the app entry, then cwd) + RFC-7396-style
> `TAURI_CONFIG` merging + `assetsArchive` and `dev` options in
> all three languages (the asset source is intentionally not env-overridable —
> the frontend is trusted content). Validated on macOS: `tauri dev` on Node/Deno/Python examples
> (window URL `http://127.0.0.1:1430/` from the built-in dev server, IPC + plugins
> working), watcher restart, `tauri build` + archive-only serving (assets dir removed,
> everything from the pack).

> **Update (2026-07-10): `tauri build` produces self-contained bundles.** `tauri build`
> on a bindings project now compiles a **true no-deps native binary** (embeds the
> language runtime) via the runner's native compiler and wraps it into real
> `.app`/`.msi`/`.appimage` through `tauri-bundler` — the same pipeline as Rust, just a
> different `AppSettings`. Output moved from `target/` to **`dist/{debug,release}/`**.
> `interface/bindings.rs::build()` detects the runner (`Runner::{Deno,Node,Python}`),
> compiles the binary (`deno compile --include . --exclude dist,tauri.conf.json`;
> PyInstaller `--onefile` honoring `PYTHONPATH` as `--paths`; Node bails — SEA can't host
> worker_threads), and stages three resources next to it: the packed `app.assets`, the
> `tauri-ffi` cdylib (located via `TAURI_FFI_LIB` or a `target/{release,debug}` search),
> and the merged config with `frontendDist` rewritten to `app.assets`.
> `get_bundle_settings` returns a `resources_map` of `dist/<profile>/resources/*` → flat
> names in the bundle resource dir (macOS `Contents/Resources`), plus icon/identifier
> from config. `get_binaries`/`app_binary_path` name the binary after `productName`.
> `bundle.rs` un-gated for bindings (generic `run_bundle::<I>`). Runtime resource
> resolution added to Deno (`bundledResourceDir()` in config.ts → config discovery +
> `libraryPath`) and Python (`_bundled_resource_dir()` via `sys.frozen`/`sys.executable`
> → config discovery + `library_path`): the compiled app reads config/lib/assets from
> `Contents/Resources` computed off the executable path. VALIDATED on macOS: `tauri build`
> on the Deno and Python examples produced `.app`s that run with **no Deno/Python on
> `PATH`** (scrubbed-PATH launch from `/tmp` — frontend served from the packed archive,
> cdylib dlopen'd from Resources, plugin ACL + IPC working), with both the debug and
> release cdylib. Rust `tauri dev` regression re-checked (dispatches to cargo via the
> trait). Not done: single-file binary (cdylib is a sibling resource, not embedded +
> extracted); Linux/Windows bundle validation; icon defaults.

> **Update (2026-07-10): Node self-contained build landed (SEA).** `tauri build` on a
> Node runner now produces a Single Executable Application. The two SEA blockers —
> one embedded CJS script (no worker files on disk) and koffi's native addon — are
> solved in `compile_node` (`interface/bindings.rs`): (1) the CLI runs the entry once
> in **trace mode** (`TAURI_SEA_TRACE=<file>`; `launch()` writes the resolved worker
> module path and exits before any FFI work — ignored when `isBundled()`), (2)
> esbuild (project-local or `npx --yes`) bundles the main entry and the worker module
> into two self-contained CJS scripts, with `--alias:koffi=` a generated shim that
> `process.dlopen`s a `koffi.node` staged as a bundle resource (the bindings only use
> koffi's native surface — `load`/`func`/`decode` — so the raw addon exports are a
> drop-in; the addon is located by a resolver script: the `@koromix/koffi-*` prebuilt
> package, else koffi's local cnoke build) and `import.meta.url` defined to the
> executable's file URL, (3) `node --experimental-sea-config` builds the blob with
> the worker bundle as the `worker.js` **SEA asset** — in a bundle, `launch()` starts
> it with `new Worker(sea.getAsset('worker.js','utf8'), { eval: true })` — and
> postject injects it into a copy of the node executable (macOS: codesign strip +
> ad-hoc re-sign). Node bindings gained `bundledResourceDir()` (config/cdylib
> resolution from `Contents/Resources`, mirroring Deno/Python). Requires Node >=
> 20.12 (checked upfront). VALIDATED on macOS: the `.app` passes the full fixture
> trace with **no node on `PATH`** and ignores malicious `TAURI_CONFIG` / `TAURI_DEV`
> / `TAURI_FFI_LIB` / `TAURI_SEA_TRACE` env (stays on `tauri://localhost`, loads only
> its bundled cdylib); `tauri dev` regression re-checked. All three runners now build
> self-contained bundles.

> **Security (2026-07-10): env overrides are dev-only.** The frontend is trusted content
> (it can `invoke` commands under the app's ACL), so its origin — and the whole config —
> must never come from the ambient environment in a shipped build. `TAURI_ASSETS_ARCHIVE`
> was removed outright (redundant with the config's `frontendDist`). `TAURI_CONFIG` and
> `TAURI_DEV` remain the Tauri CLI→app channel in dev but are now **ignored by a compiled/
> frozen binary**: `isBundled()` (Deno `Deno.build.standalone`, Python `sys.frozen`, Node
> `require('node:sea').isSea()`) gates the `TAURI_CONFIG` merge and the `TAURI_DEV`
> default. Explicit `config`/`dev`/`assetsArchive` launch options are code (trusted) and
> still honored. VALIDATED: a compiled Deno/Python `.app` launched with a malicious
> `TAURI_CONFIG` (frontendDist/devUrl → `http://evil.test`) + `TAURI_DEV=true` ignored
> both — stayed on `tauri://localhost` serving its bundled assets — while `tauri dev`
> still picked up the CLI's `TAURI_CONFIG` dev-server URL. `TAURI_FFI_LIB` (which cdylib
> to `dlopen`) is gated the same way — a bundle loads only its own Resources cdylib,
> verified by launching the Deno and Python `.app`s with
> `TAURI_FFI_LIB=/nonexistent/evil.dylib` (ignored; loaded the bundled lib). So every
> ambient-env override is now dev-only; only explicit launch-option arguments (code) can
> steer a shipped bundle.

**M3 — Manifest + codegen + Deno.** Decided and partially landed early (§6, 2026-07-09):
hand-authored manifest + in-house generator now produce the C header, koffi
declarations, Deno symbol map and Python cffi cdef, with a lib.rs drift guard and a
`--check` CI gate; the Node, Deno and Python packages all run on generated artifacts
(see M2 status). Remaining: generate the Rust extern shims from the same manifest
(M3a), shared TS high-level layer + generated docs (M3b). Manifest diff reviewed on
every PR.

**M4 — Surface growth + distribution.** Tray/menu/dialog via plugin features; prebuilt
binaries for {macOS, Windows, Linux} × {x64, arm64} published per release; npm/JSR/PyPI
publishing pipelines; docs (generated API reference + per-language guides); ABI
versioning policy in effect. From here, surface expansion is routine one-PR work.

> **Status (2026-07-09): distribution pipeline landed** —
> `.github/workflows/publish-ffi.yml` builds the cdylib once per target
> ({macOS, Windows, Linux} × {x64, arm64}) and fans out to every channel from the
> same artifacts (vs. publish-cli-js.yml where each language would need its own
> native build): C archives (header + lib + licenses + SHA256SUMS) attached to a
> GitHub release via `bindings/scripts/dist.mjs`; npm `@tauri-apps/node` +
> per-platform optionalDependencies packages generated at publish time (loader
> resolves them first, falls back to the repo dev build); PyPI `tauri-ffi`
> py3-none platform wheels bundling the lib in `tauri_ffi/_native/` (Linux tagged
> manylinux with the documented webkit2gtk runtime requirement); JSR
> `@tauri-apps/deno` with a first-run download of the raw lib asset from the
> GitHub release into a per-version cache. Gates: headless FFI smoke test on 4
> platforms (`bindings/scripts/smoke.mjs`) + `generate.mjs --check`. All
> publishes use OIDC trusted publishing (registry-side configuration required);
> runs default to dry-run. Deferred: covector integration once the packages
> stabilize, Windows C example linking, macOS codesigning of the dylib.

---

## 9. Risks & open questions

1. **`Context::new` / `RuntimeAuthority::new` are declared unstable.** Mitigated by
   living in-tree; consider promoting a supported `tauri::runtime-context` constructor
   (feature-gated) as part of M1 so out-of-tree embedders benefit too.
2. **ACL manifest embedding mechanism** — reusing tauri-build's collection from a plain
   crate build script needs confirmation (M0 item). Fallback: vendor the core permission
   manifests as JSON in `tauri-ffi` (they're in-repo).
3. **Node worker bootstrap DX** — the "your app runs in a worker" model needs to feel
   invisible (`launch()` handles it). Long-term fix is tao `pump_events` (§3 roadmap bet);
   design the Node sugar so a future no-worker mode is a non-breaking swap.
4. **Windows deadlock** creating windows synchronously inside handlers
   (`webview_window.rs:115`) — the queue model naturally avoids it (host reacts on its
   own thread); direct-mode docs must warn loudly.
5. **Config `deny_unknown_fields`** makes host JSON brittle across tauri versions —
   surface precise serde errors through `tauri_last_error_message`; consider a
   `tauri_config_validate(json)` helper.
6. **Linux runtime deps** (webkit2gtk-4.1 etc.) can't be statically linked — document
   per-distro requirements in each package README; matches existing tauri app reality.
7. **Callback discipline** — hosts throwing/panicking inside callbacks, or blocking the
   main thread inside a run-event callback. Contract: callbacks must not block; violations
   are a documented foot-gun, and queue mode (no reentry) is the recommended default for
   GC'd languages.
8. **Not in scope v0:** mobile targets, custom `Runtime` impls, isolation pattern
   (`Pattern::Brownfield` only), servo/verso backend.
