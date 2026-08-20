---
"tauri": patch:bug
---

Fixed a potential deadlock where synchronous menu commands (e.g. `Menu.new()`) held the plugin store lock while blocking on a main-thread hop, while the main thread itself needed that same lock to process a queued event. The affected `menu` plugin commands are now dispatched asynchronously so the lock is released before they hop to the main thread.
