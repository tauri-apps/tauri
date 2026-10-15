---
"tauri": major:breaking
---

Removed the deprecated `Invoke::state` and `Invoke::state_ref` methods; use `Manager::state` on `Invoke::webview_ref()` instead. `tauri::StateManager` is no longer exported.
