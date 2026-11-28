---
"tauri-runtime-cef": patch:bug
---

On Linux, Made `WindowMessage::StartDragging` and `WindowMessage::StartResizeDragging` to request the WM to start dragging/resizing since the child window never recieves the pointer button press.
