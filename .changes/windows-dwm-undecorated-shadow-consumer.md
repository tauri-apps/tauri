---
"tauri": patch:enhance
"tauri-runtime": patch:enhance
"tauri-runtime-wry": patch:enhance
---

On Windows, consume tao's DWM-anchored shadow for undecorated windows: the client area of an undecorated shadowed window equals the window rect, a symmetric border is drawn on every edge (Windows 10's top border included), and the webview content keeps the frame inset the border needs there. Adds `WindowBuilder::undecorated_border_transparent` to keep the shadow while drawing no visible border.
