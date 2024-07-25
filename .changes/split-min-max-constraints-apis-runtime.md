---
"tauri": patch:feat
---

Add APIs to enable setting window size constraints separately:
- Added `WindowBuilder::inner_size_constraints` and `WebviewWindowBuilder::inner_size_constraints` which can be used for setting granular constraints.
- Added `WindowSizeConstraints` struct
- Added `Window::set_size_constraints` and `WebviewWindow::set_size_constraints`
