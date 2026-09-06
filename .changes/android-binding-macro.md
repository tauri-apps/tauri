---
"tauri": major:breaking
"tauri-macros": major:breaking
"tauri-runtime-wry": minor:feat
---

The `tauri::android_binding!` macro moved to `tauri_runtime_wry::android_binding!`, and `#[tauri::mobile_entry_point]` expands to it, so Android apps must depend on `tauri-runtime-wry`. `tauri::handle_android_plugin_response` and `tauri::send_channel_data` are exposed on Android for other runtimes to implement their own binding.
