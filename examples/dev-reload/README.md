# Experimental state-preserving Rust reload

The desktop debug host retains its window, WebView and JSON business state.
Only an opt-in business worker is rebuilt. This does not transparently reload
existing `#[tauri::command]` functions, plugins, setup, or native window code.

From the repository root, start the watcher in a separate terminal:

```sh
python3 scripts/dev-reload.py --manifest examples/dev-reload/backend/Cargo.toml \
  --bin reload-counter --output examples/dev-reload/backend/.reload
# To use justrust's local builder, add --runner justrust --runner-arg=--local.
```

In another terminal, run the included window example:

```sh
cargo run -p tauri --example dev-reload
```

Type in the input, increment the counter, then edit the label in
`backend/src/main.rs`. The displayed label updates after compilation without
clearing the input or counter. Source changes to the host itself require a full
restart; the separate worker watcher deliberately watches only business code.

In a desktop app using this checkout of Tauri, wire this development command:

```rust,ignore
#[cfg(debug_assertions)]
#[tauri::command]
async fn reload_call(
  host: tauri::State<'_, std::sync::Arc<std::sync::Mutex<tauri::dev_reload::ReloadHost>>>,
  command: String,
  payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
  let host = host.inner().clone();
  tauri::async_runtime::spawn_blocking(move || {
    host.lock().map_err(|e| e.to_string())?.call(&command, payload)
  }).await.map_err(|e| e.to_string())?
}

// Add to your existing builder only in desktop debug builds:
let builder = builder
  .manage(std::sync::Arc::new(std::sync::Mutex::new(
    tauri::dev_reload::ReloadHost::new(
      "/absolute/path/to/backend/.reload/current",
      serde_json::json!({"count": 0}),
      std::time::Duration::from_secs(3),
    ),
  )))
  .invoke_handler(tauri::generate_handler![reload_call]);
```

Invoke `reload_call` with `{command: "increment", payload: null}`. Change the
worker label and save. After publication, the next call returns the new label
and continues counting without recreating the window or reloading the page.
Add `--watch /path/to/local/dependency` for sources outside the backend directory.

Protocol v1 is newline-delimited JSON over stdin/stdout. The worker first writes
`{"protocol":1}`, then receives `{command,payload,state}` and returns
`{result,state}`. Reserve stdout for protocol; log to stderr. Keep state schema
compatible, or explicitly migrate it in the worker. Process-local memory is not
preserved. The host serializes calls and retains the last committed state after
worker failure. External side effects cannot be rolled back. Do not hold native
resources in the worker that require identity across reloads.

Build failure leaves `current` untouched. New binaries have unique paths; failed
handshakes retain the previous live worker. A request failure kills the worker
and rejects that call; the next call starts from committed state. Requests are
never automatically retried. Old generations remain until the developer removes
`.reload` after closing all hosts. Keep this directory outside source control.
This API is compiled only for desktop debug builds. Release apps must register
normal commands and must not start the watcher.

## Reference

Design reference: the locally installed `1jehuang/justrust` source,
revision `1bf0b832a30926a3f8a4a78a6e0d349397a97444`,
`bench/hotpatch/README.md`, `loader.c`, and `docs/toolchain.md`.
That prototype patches changed CGUs before a fresh test process enters `main`;
it is not live GUI hot reload. We reuse the ideas of a stable base, replaceable
artifacts and fallback, with a versioned process boundary instead of patching
machine code. No justrust implementation is copied. This does not implement its
compiler graph/cache or claim its build performance.

## Validation

```sh
cargo test --offline --manifest-path examples/dev-reload/backend/Cargo.toml
python3 examples/dev-reload/test_watcher.py
# Linux, after building the dev-reload example:
python3 examples/dev-reload/test_webview.py
```

The tests compile the actual host module independently of GUI dependencies and
exercise worker replacement, state continuity, rejected handshakes, timeout,
malformed responses, real Cargo rebuilds, compile failure and recovery. A local
`justrust build --local` publication was also verified. After fetching the pinned
dependencies, both `cargo check -p tauri --no-default-features` and
`cargo build -p tauri --example dev-reload` passed. The real-WebView acceptance
test cannot execute in the agent's isolated display environment. On 2026-11-12,
the user ran it from a normal local terminal: it reached `RELOAD_SMOKE:ready`,
rebuilt and published the edited worker, then reached `RELOAD_SMOKE:passed` and
exited successfully. This verifies that the real WebView retained its document,
input and Rust counter across the rebuild. That run reported a 0.08-second
build-and-publication step; this is one observation, not an end-to-end latency
benchmark. Mesa DRI3 warnings did not prevent the test from passing.
