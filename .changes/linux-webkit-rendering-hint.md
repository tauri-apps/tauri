---
"tauri-runtime-wry": patch:enhance
---

On Linux and the BSDs, log a one-time warning naming the `WEBKIT_DISABLE_DMABUF_RENDERER=1` / `WEBKIT_DISABLE_COMPOSITING_MODE=1` workarounds when software GL rendering is detected via the environment (`LIBGL_ALWAYS_SOFTWARE`, `GALLIUM_DRIVER`), plus a debug-level breadcrumb on every runtime initialization.
WebKitGTK renders a blank window in such environments with no diagnostic of its own.
Log-only; no environment variables are modified.
