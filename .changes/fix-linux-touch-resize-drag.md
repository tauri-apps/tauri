---
"tauri": patch:bug
"tauri-runtime-wry": patch:bug
---

Fix a Linux session crash where every touch event near the border of an undecorated window started another resize drag. A resize drag now only starts on touch begin.
