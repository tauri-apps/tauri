---
'tauri': 'patch:breaking'
---

`onNewIntent` is no longer called on the `onCreate` hook if the intent is set. Use the `onCreate` hook to handle it.
