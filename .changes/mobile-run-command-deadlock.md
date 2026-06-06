---
"tauri": patch:bug
---

Adjust mutex locking in `send_channel_data_handler`, `handle_android_plugin_response`, `send_channel_data` to avoid deadlocks
