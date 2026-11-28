---
"tauri": patch:bug
---

Fix the isolation pattern hanging every IPC call when the CSP sets `frame-src` or `child-src`. The isolation frame origin is now added to those directives too, not only to `default-src`.
