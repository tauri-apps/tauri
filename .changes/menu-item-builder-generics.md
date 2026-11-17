---
"tauri": "patch:breaking"
---

Added `<R: Runtime>` generic on `MenuItemBuilder`, `CheckMenuItemBuilder` and `IconMenuItemBuilder` and removed it from `build` method.

The builders default to `DynRuntime`.
