---
"tauri": "major:breaking"
---

Represent `ipc::Channel<T>` as an opaque Specta reference whose state is `T`'s datatype instead of the synthetic `TAURI_CHANNEL` named type. Binding generators must downcast the opaque reference to `specta::datatype::DataType` and render the channel from it.
