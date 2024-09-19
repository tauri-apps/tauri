---
"tauri-utils": patch:breaking
"tauri": patch:breaking
---

The `Assets::iter` function now must return a iterator with `Item = (Cow<'_, str>, Cow<'_, [u8]>)` to be more flexible on contexts where the assets are not `'static`.
