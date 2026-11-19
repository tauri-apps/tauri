---
"tauri": "major:breaking"
---

Represent the synthetic `TAURI_CHANNEL` Specta type with a sealed, opaque body instead of a struct. The named type still carries `T` as a generic so binding generators can render `Channel<T>`, but the body can no longer be constructed outside Tauri.
