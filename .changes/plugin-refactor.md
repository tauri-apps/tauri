---
"tauri": patch
---

Refactored the `Plugin` trait `initialize` and `extend_api` signatures.
`initialize` now takes the `App` as first argument, and `extend_api` takes a `InvokeResolver` as last argument.
This adds support to managed state on plugins.
