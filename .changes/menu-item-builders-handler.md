---
"tauri": "minor:feat"
---

Add `handler` method on `MenuItemBuilder`, `CheckMenuItemBuilder` and `IconMenuItemBuilder`. Handlers receive a temporary reference to the activated item without keeping it alive and are removed when the last reference to the item is dropped.
