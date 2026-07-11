# API example (Python)

Runs Tauri's [`examples/api`](../../../../examples/api) validation app — the
same Svelte frontend the Rust example uses — with a **Python** host instead of a
Rust backend.

The frontend is **not** copied here. `tauri.conf.json` references the shared
app directly:

- `beforeDevCommand` runs `pnpm dev` in `examples/api` (the Vite dev server),
  and `devUrl` points the window at it during `tauri dev`.
- `beforeBuildCommand` / `frontendDist` point at `examples/api` /
  `examples/api/dist` for `tauri build`.

[`main.py`](main.py) supplies the host side — the `log_operation`,
`perform_request`, `echo`, `spam` commands and the `app-menu` plugin the
frontend invokes (see `examples/api/src-tauri/src`).

Permissions come from [`capabilities/run-app.json`](capabilities/run-app.json),
auto-discovered next to the config the same way `tauri-build` discovers a Rust
app's `capabilities/` directory at compile time.

## Running

From the repo root, build the frontend deps once (`pnpm i && pnpm build`), then
from this directory:

```sh
pip install cffi
cargo build -p tauri-cli
../../../../target/debug/cargo-tauri dev
```
