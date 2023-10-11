---
'tauri': 'major:breaking'
---

The event system APIS on Rust is recieving a few changes for consistency and quality of life improvements:

- Renamed `Manager::emit_all` to just `Manager::emit` and will now both trigger the events on JS side as well as Rust.
- Removed `Manager::trigger_global`, use `Manager::emit`
- Added `Manager::emit_filter`.
- Removed `Window::emit`, and moved the implementation to `Manager::emit`.
- Removed `Window::emit_and_trigger` and `Window::trigger`, use `Window::emit` instead.
- Changed `Window::emit_to` to only trigger the target window listeners so it won't be catched by `Manager::listen_global`
