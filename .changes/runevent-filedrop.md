---
'tauri': 'patch:bug'
'tauri-runtime-wry': 'patch'
---

Fix `RunEvent::WindowEvent(event: WindowEvent::FileDrop(FileDropEvent))` never triggered and always prevent default OS behavior when `disable_file_drop_handler` is not used.
