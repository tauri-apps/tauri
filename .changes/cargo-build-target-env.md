---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Respect the `CARGO_BUILD_TARGET` environment variable when resolving the build target, matching Cargo's precedence over `build.target` in `.cargo/config.toml`.
