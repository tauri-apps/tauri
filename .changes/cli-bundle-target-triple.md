---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix the bundler using the host target triple when `--target` is not passed but `build.target` is set in `.cargo/config.toml`. The CLI now also accepts `build.target` as an array and honors the `CARGO_BUILD_TARGET` environment variable.
