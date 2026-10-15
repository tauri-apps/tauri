---
tauri-runtime: patch:bug
tauri-runtime-wry: patch:bug
tauri-runtime-cef: patch:bug
---

Fix runtime crates failing to compile when feature unification enables `tauri-runtime/macos-private-api` but not the runtime crate's own `macos-private-api` feature. `WindowBuilder::transparent` is no longer gated on the feature; on macOS, runtimes implement it as a no-op unless their own `macos-private-api` feature is enabled, so the private API is never referenced without it.
