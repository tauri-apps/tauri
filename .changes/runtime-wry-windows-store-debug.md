---
'tauri-runtime-wry': 'patch:bug'
---

`WindowsStore` and `ActiveTraceSpanStore` now implement `Debug` manually without borrowing the inner `RefCell`, so formatting them re-entrantly while the store is already borrowed no longer panics with "already borrowed".
