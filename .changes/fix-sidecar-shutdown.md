---
"tauri": patch
---

Emits RunEvent::Exit prior to killing child processes managed by tauri.
Allows for graceful shutdown of sidecar binaries.
