---
"tauri": "patch:bug"
"tauri-runtmie-wry": "patch:bug"
---

On Windows, handle resizing undecorated windows natively which improves performance and fixes a couple of annoyances with previous JS implementation:
- No more cursor flickering when moving the cursor across an edge.
- Can resize from top even when `data-tauri-drag-region` element exists there.
- Upon starting rezing, clicks don't go through elements behind it so no longer accidental clicks.

