---
'tauri-runtime-wry': 'patch:enhance'
---

Print a one-line advisory at startup when running under Wayland with webkit2gtk 2.44.x, naming `WEBKIT_DISABLE_DMABUF_RENDERER=1` as the workaround for the upstream DMA-BUF renderer launch failure (`Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`, see tauri-apps/tauri#10702 and #9304).
