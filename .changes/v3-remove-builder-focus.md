---
"tauri": major:breaking
---

Removed the deprecated `WindowBuilder::focus` and `WebviewWindowBuilder::focus` methods. Windows are focused by default; use `focused(bool)` to change that.
