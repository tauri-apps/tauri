---
'tauri': 'patch:bug'
---

`once` no longer panics when a queued emit is replayed before the handler's queued unlisten is processed (#16214). The handler is consumed before being called, so a replayed dispatch that reaches the same listener again is a no-op instead of tripping the once-guard panic.
