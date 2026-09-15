---
"tauri-bundler": major:breaking
---

`tauri_bundler::Error::Context` now wraps a `Box<dyn std::error::Error + Send + Sync + 'static>` instead of a `Box<tauri_bundler::Error>`, and the `Context` trait is implemented for any `Result<T, E: std::error::Error + Send + Sync + 'static>`. The unused `Error::BundlerError(anyhow::Error)` and deprecated `Error::BinaryOffsetOutOfRange` variants were removed, and the crate no longer depends on `anyhow`.
