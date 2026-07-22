# tauri-ffi language bindings (experimental)

Language bindings to Tauri over the `crates/tauri-ffi` C ABI — see
[/ffi-bindings-plan.md](../ffi-bindings-plan.md). One hand-designed C ABI, described by
`crates/tauri-ffi/api-manifest.json`; every per-language declaration layer is generated
from it by `bindgen/generate.mjs` (`--check` in CI). No napi-rs, no pyo3, no
node-gyp — every package is pure script + the prebuilt cdylib.

| directory            | consumer     | runtime                                        |
| -------------------- | ------------ | ---------------------------------------------- |
| [`c/`](c/)           | C            | header + your compiler                         |
| [`node/`](node/)     | Node.js ≥ 18 | [koffi](https://koffi.dev)                     |
| [`deno/`](deno/)     | Deno 2       | built-in `Deno.dlopen`                         |
| [`python/`](python/) | Python 3     | [cffi](https://cffi.readthedocs.io) (ABI mode) |

## Running the examples

Use each example's launcher to run it — it stages the freshly built native
library into the binding's `_native/` and then invokes the Tauri CLI:

```sh
cargo build -p tauri-cli
cd bindings/node/examples/hello
node ../../../scripts/dev.mjs dev
```

## Tauri CLI (`tauri dev` / `tauri build`)

Bindings projects are full Tauri CLI projects: a `tauri.conf.json` with a
`build > runner` command makes `tauri dev` and `tauri build` work like they do for
Rust apps (the CLI detects non-cargo projects by the missing `Cargo.toml`):

```jsonc
// tauri.conf.json
{
  "build": {
    "runner": { "cmd": "node", "args": ["main.js"] },
    "frontendDist": "./assets"
  }
}
```

- `tauri dev` runs the `beforeDevCommand`, serves `frontendDist` on the built-in dev
  server (or waits for your `devUrl`), spawns the runner and restarts it on file
  changes. The fully merged config reaches the app through `TAURI_CONFIG`, and
  `TAURI_DEV=true` makes windows load from the dev URL — no Rust toolchain involved.
- `tauri build` compiles the app into a **self-contained native binary** with the
  runner's native compiler (`deno compile`, PyInstaller, Node Single Executable
  Applications), so it runs on a machine without the language runtime installed —
  just like a Rust `tauri build`. It stages the packed frontend assets, the
  `tauri-ffi` cdylib, the config and the `capabilities/` directory as bundle
  resources next to the binary, then hands everything to `tauri-bundler` for
  `.app`/`.msi`/`.appimage` packaging. Output goes under `dist/` (not `target/`).

  ```sh
  tauri build                       # -> dist/release/bundle/macos/<App>.app
  ```

  The compiled app finds its resources next to the executable (the bundle's resource
  dir), so both `dist/release/<App>` and the packaged bundle run standalone. Set
  `TAURI_FFI_LIB` to point the CLI at the cdylib to embed.

  The Node build needs Node >= 20.12 and uses `esbuild` and `postject`, which ship
  as dependencies of `@tauri-apps/node` (the CLI resolves them from
  `node_modules/.bin`, falling back to `npx --yes`): the CLI runs your entry once in
  trace mode to learn the worker module, bundles both into self-contained CJS
  scripts (the worker rides along as a SEA asset and is started from source via
  `new Worker(code, { eval: true })`), and injects the blob into a copy of the node
  executable. koffi's native addon can't live inside the executable, so it is
  staged as a bundle resource and dlopen'd from there.

`launch()` (and Python's `App()`) auto-discovers `tauri.conf.json` next to the app
entry, so the same project runs identically with `node main.js` or `tauri dev`.

## Capabilities

Like a Rust app, a bindings project declares its ACL in a `capabilities/` directory
next to the config. `launch()` / `App()` reads every `*.json`/`*.json5`/`*.toml`
file there (a `schemas/` subfolder is ignored) and applies it — the runtime
equivalent of the compile-time discovery `tauri-build` does for a Rust app.
`tauri build` stages the directory into the bundle resources, so a compiled binary
keeps the exact same grants; there is nothing to embed at compile time because the
`tauri-ffi` cdylib is app-agnostic. When no `capabilities/` directory exists, the
app falls back to granting `core:default` to every window. Inline capabilities
passed in code (`launch({ capabilities })`) are merged on top.

## Publishing

`.github/workflows/publish-ffi.yml` builds every target and publishes all packages
(dry-run by default).

PyPI and JSR trusted publishers are configured on the respective registry sites.

## Regenerating after an ABI change

```sh
node bindings/bindgen/generate.mjs         # rewrite generated artifacts
node bindings/bindgen/generate.mjs --check # CI staleness gate
```
