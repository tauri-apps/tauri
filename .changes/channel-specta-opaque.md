---
"tauri": "major:breaking"
---

Represent `ipc::Channel<T>` as an opaque Specta reference containing `ipc::ChannelType` instead of the synthetic `TAURI_CHANNEL` named type. Binding generators must downcast the opaque reference to `ChannelType` and use its `message` datatype.
