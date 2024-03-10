---
"tauri": patch:breaking
---

The `Context` type no longer uses the `<A: Assets>` generic so the assets implementation can be swapped with `Context::assets_mut`.
