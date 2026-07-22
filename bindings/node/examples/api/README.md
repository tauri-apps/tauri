# API example (Node.js)

Runs Tauri's [`examples/api`](../../../../examples/api) validation app — the
same Svelte frontend the Rust example uses — with a **Node.js** host instead of
a Rust backend.

The frontend is **not** copied here. `tauri.conf.json` references the shared
app directly:

- `beforeDevCommand` runs `pnpm dev` in `examples/api` (the Vite dev server),
  and `devUrl` points the window at it during `tauri dev`.
- `beforeBuildCommand` / `frontendDist` point at `examples/api` /
  `examples/api/dist` for `tauri build`.

[`app.js`](app.js) supplies the host side — the `log_operation`,
`perform_request`, `echo`, `spam` commands and the `app-menu` plugin the
frontend invokes (see `examples/api/src-tauri/src`).

Permissions come from [`capabilities/run-app.json`](capabilities/run-app.json),
auto-discovered next to the config the same way `tauri-build` discovers a Rust
app's `capabilities/` directory at compile time.

## Running

From the repo root, build the frontend deps once (`pnpm i && pnpm build`), then
from this directory:

```sh
cargo build -p tauri-cli

# wry — the OS webview (default)
node ../../../scripts/dev.mjs dev

# cef — Chromium Embedded Framework (linux only for now)
TAURI_RUNTIME=cef node ../../../scripts/dev.mjs dev
```

[`dev.mjs`](../../../scripts/dev.mjs) is a thin wrapper around the freshly built
`cargo-tauri`. Swap `dev` for `build` to bundle instead. `TAURI_RUNTIME` is the single
webview switch: it sets `app.runtime` (via `--config`) so the CLI loads the
matching `libtauri_<runtime>`, and the same value reaches `stage-dev.mjs` in
`beforeDevCommand` so that library gets staged. Leave it unset to use the
config's own `app.runtime`.
