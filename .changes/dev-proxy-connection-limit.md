---
"tauri": patch:bug
---

Fix the mobile dev proxy returning HTTP 500 on cold starts with large module graphs: each proxied request opened its own TCP connection to the dev server and exceeded its accept backlog, so the kernel reset the excess connections (`ECONNRESET`). A semaphore now bounds in-flight proxy requests to 32.
