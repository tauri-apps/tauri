---
"tauri-macros": patch
---

`#[command]` now generates a macro instead of a function to allow passing through `Params` and other generics.
`generate_handler!` has been changed to consume the generated `#[command]` macro
