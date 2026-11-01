---
tauri-bundler: 'patch:bug'
---

Drop libEGL and libGLESv2 DLLs from bundling as they're now statically linked in CEF and not distributed.
