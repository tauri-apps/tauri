---
'tauri-runtime-cef': 'patch:bug'
---

Tidying up the context menu's leftover separators can no longer hang the application. The cleanup loops retried `MenuModel::remove_at` without checking whether it succeeded, and a removal CEF refuses does not shrink the model, so a single refusal spun forever on CEF's UI thread — which this runtime drives with an external message pump, so the whole app would stop responding rather than just the menu. The loops now advance past an entry they could not remove.
