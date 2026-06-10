---
"tauri-runtime-wry": "patch:bug"
---

Fix a `RefCell` `BorrowMutError` panic on mobile: the `Resumed`/`Suspended` event branch held a `windows` borrow across the window-event handlers and the `RunEvent` callback, so any of them that created or closed a window (e.g. from a resume/suspend handler) panicked.
