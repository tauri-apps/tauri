---
"cli.rs": patch
"tauri-bundler": patch
"tauri-utils": patch
"tauri-build": patch
---

Move the copying of resources and sidecars from `cli.rs` to `tauri-build` so using the Cargo CLI directly processes the files for the application execution in development.
