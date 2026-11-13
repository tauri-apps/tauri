---
'tauri-runtime-wry': 'patch:enhance'
---

On Linux, Wayland sessions with webkit2gtk 2.44.x can be killed at launch by an upstream WebKitGTK bug: the app dies with `Error 71 (Protocol error) dispatching to Wayland display` before any window shows (#10702, #9304). The runtime now prints a heads-up at startup in that situation, telling the user to relaunch with `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
