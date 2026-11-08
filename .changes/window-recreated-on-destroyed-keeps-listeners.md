---
"tauri": patch:bug
---

Fixed a window recreated under the same label while the previous one is being destroyed (from a `WindowEvent::Destroyed` handler or a plugin's `on_event`) losing the event listeners registered on it in the meantime.
