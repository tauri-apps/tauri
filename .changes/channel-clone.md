---
"tauri": "patch:bug"
---

Removed `TSend: Clone` requirement for `Channel<TSend>` by implementing `Clone` manually instead of driving it.
