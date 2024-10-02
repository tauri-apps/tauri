---
"@tauri-apps/cli": "patch:feat"
---

Improved support for `dpi` module types to allow these types to be used without manual conversions with `invoke`:

- Added `toIPC`  method on `PhysicalSize`, `PhysicalPosition`, `LogicalSize` and `LogicalPosition` to convert it into a valid IPC-compatible value that can be deserialized correctly on the Rust side into its equivalent struct.
- Implemented `toJSON`  method on `PhysicalSize`, `PhysicalPosition`, `LogicalSize` and `LogicalPosition` that calls the new `toIPC` method, so `JSON.stringify` would serialize these types correctly.
