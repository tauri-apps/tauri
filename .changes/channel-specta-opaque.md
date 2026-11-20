---
"tauri": "major:breaking"
---

Represent the synthetic `TAURI_CHANNEL` Specta type with a sealed, opaque body instead of a struct. The named type still carries `TSend` as a generic so binding generators can render `Channel<TSend>`, but the body is a private opaque marker that cannot be named or constructed outside Tauri.

This aligns `Channel` with how `ipc::Request` and `ipc::Response` are implemented. At the time we added `impl Type for Channel`, opaque types didn't exist hence we didn't use them.
