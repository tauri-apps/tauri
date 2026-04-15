---
"tauri-utils": minor:deps
---

Add new `html-manipulation-2` and `build-2` feature flags that use `dom_query` instead of `kuchikiki` for HTML parsing / manipulation.
This allows downstream users to remove `kuchikiki` and its dependencies from their dependency tree.
