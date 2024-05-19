---
"tauri-runtime-wry": patch:bug
---

Multiwebview mode no longer relies on wry's `WebViewBuilder::new_gtk` since `new_as_child` is more stable on initial size computation. This breaks window menus until `muda` offers support for the multiwebview GTK container.
