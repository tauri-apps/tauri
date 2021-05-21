---
"tauri": patch
---

Adds the default types used with `Builder::default()` to items that expose `Params` in their type. This allows you to
skip specifying a generic parameter to types like `Window<P>` if you use the default type.
