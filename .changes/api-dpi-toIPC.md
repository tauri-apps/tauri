---
"@tauri-apps/api": "minor:feat"
"tauri": "minor:feat"
---

Improved support for `dpi` module types to allow these types to be used without manual conversions with `invoke`:

- Added `SERIALIZE_TO_IPC_FN` const in `core` module which can be used to implement custom IPC serialization for types passed to `invoke`.
- Added `Size` and `Position` classes in `dpi` module.
- Implementd `SERIALIZE_TO_IPC_FN` method on `PhysicalSize`, `PhysicalPosition`, `LogicalSize` and `LogicalPosition` to convert it into a valid IPC-compatible value that can be deserialized correctly on the Rust side into its equivalent struct.
