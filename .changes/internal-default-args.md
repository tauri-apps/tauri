---
"tauri": patch
---

(internal): allow `wry` dependency to be optional again while keeping default args.
code that wishes to expose a struct with a default arg should use the `crate::manager::default_args!` macro to declare
the struct, so that it can automatically feature-gate `DefaultArgs` behind using `wry`.
