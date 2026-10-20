---
"tauri-runtime-wry": patch:enhance
---

On Windows, draw the shadow of undecorated (frameless) windows through DWM instead of tao's undecorated-shadow emulation, so the client area matches the window rect without hidden native frame insets. Transparent windows keep the system default shadow behavior.
