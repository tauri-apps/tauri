---
"tauri": major:breaking
---

`Plugin::initialization_script` now returns `Option<InitializationScript>` (re-exported as `tauri::webview::InitializationScript`) instead of `Option<String>`, replacing the interim `Plugin::initialization_script_2`. Set `for_main_frame_only: false` to also run the script on sub frames.
