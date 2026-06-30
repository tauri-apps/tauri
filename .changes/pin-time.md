---
tauri: patch:deps
---

Pinning `time` to `<0.3.52` used by `cookie` to mitigate a compilation error, see

- https://github.com/tauri-apps/tauri/issues/15615
- https://github.com/rwf2/cookie-rs/issues/255
