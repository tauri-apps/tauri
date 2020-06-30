---
"tauri-bundler": minor
---

The bundler now bundles all binaries from your project ([[[bin]] target tables](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)) and [src/bin folder](https://doc.rust-lang.org/cargo/guide/project-layout.html).
When multiple binaries are used, make sure to use the [default-run](https://doc.rust-lang.org/cargo/reference/manifest.html#the-default-run-field) config field.
