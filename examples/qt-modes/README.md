# Qt modes example

Runs one set of IPC checks ([`ui/checks.js`](./ui/checks.js)) on
[`tauri-runtime-qt`](../../crates/tauri-runtime-qt) under each Tauri mode:

| Mode          | Pattern    | Page served from                                                                                    | IPC transport                     |
| ------------- | ---------- | --------------------------------------------------------------------------------------------------- | --------------------------------- |
| `brownfield`  | brownfield | `tauri://localhost`, CSP with nonces                                                                | `ipc://` protocol                 |
| `isolation`   | isolation  | `tauri://localhost` + isolation iframe                                                              | `ipc://`, via the iframe          |
| `devserver`   | brownfield | `devUrl` (`http://localhost:17421`)                                                                 | `ipc://` protocol                 |
| `postmessage` | brownfield | `tauri://localhost`, CSP without `ipc:`                                                             | `window.ipc.postMessage` fallback |
| `noglobal`    | brownfield | `tauri://localhost`, no CSP, no `window.__TAURI__`                                                  | `ipc://` protocol                 |
| `remote`      | brownfield | `http://127.0.0.1:17422` with a `remote` capability, plus a page without one (IPC must be rejected) | `ipc://` protocol                 |

The configuration of each mode is in [`src-tauri/modes`](./src-tauri/modes) (the isolation
mode uses [`src-tauri/tauri.conf.json`](./src-tauri/tauri.conf.json) and
[`isolation-dist`](./isolation-dist)).

## Running the example

From the root of the repository:

```bash
QT_MODES_MODE=isolation cargo run -p qt-modes   # one mode
cargo test -p qt-modes                          # every mode, offscreen
```

Each check prints `[PASS]`/`[FAIL]`; the app exits with 0 if all passed, 1 if some failed and
2 if the page never finished (e.g. IPC is broken).
