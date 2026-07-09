# tauri-ffi language bindings (experimental)

Language bindings to Tauri over the `crates/tauri-ffi` C ABI — see
[/ffi-bindings-plan.md](../ffi-bindings-plan.md). One hand-designed C ABI, described by
`crates/tauri-ffi/api-manifest.json`; every per-language declaration layer is generated
from it by `bindgen/generate.mjs` (`--check` in CI). No napi-rs, no pyo3, no
node-gyp — every package is pure script + the prebuilt cdylib.

| directory | consumer | runtime |
|---|---|---|
| [`c/`](c/) | C | header + your compiler |
| [`node/`](node/) | Node.js ≥ 18 | [koffi](https://koffi.dev) |
| [`deno/`](deno/) | Deno 2 | built-in `Deno.dlopen` |
| [`python/`](python/) | Python 3 | [cffi](https://cffi.readthedocs.io) (ABI mode) |

## Running the examples

```sh
node bindings/run-example.mjs --list     # what's available
node bindings/run-example.mjs node       # bindings/node/examples/hello
node bindings/run-example.mjs python
node bindings/run-example.mjs deno
node bindings/run-example.mjs c
```

The runner builds `tauri-ffi`, handles per-language prerequisites (koffi install via
pnpm, cffi check, C compilation) and runs the example. Any example path works too:
`node bindings/run-example.mjs bindings/python/examples/hello`.

Environment: `TAURI_FFI_LIB` (library override), `PYTHON` (interpreter, default
`python3`), `CC` (C compiler, default `cc`), `FIXTURE_STATUS` (examples append their
smoke-test trace to this file).

## Publishing

`.github/workflows/publish-ffi.yml` builds every target and publishes all packages
(dry-run by default).

PyPI and JSR trusted publishers are configured on the respective registry sites.

## Regenerating after an ABI change

```sh
node bindings/bindgen/generate.mjs         # rewrite generated artifacts
node bindings/bindgen/generate.mjs --check # CI staleness gate
```
