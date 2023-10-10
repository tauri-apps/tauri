---
'tauri': 'major:breaking'
---

The event system APIS on Rust is recieving a few changes for consistency and quality of life improvements:

- Renamed `Manager::emit_all` to just `Manager::emit` and will now both trigger the events on JS side as well as Rust.
- Removed `Manager::trigger_global`, use `Manager::emit`
- Added `Manager::emit_filter`.
- Changed `Window::emit` to trigger the events on the Rust side as well.
- Removed `Window::emit_and_trigger` and `Window::trigger`, use `Window::emit` instead.
