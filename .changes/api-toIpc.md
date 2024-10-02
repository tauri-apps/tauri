---
"@tauri-apps/cli": "patch:feat"
---

Add `toIpc` method on `PhysicalSize`, `PhysicalPosition`, `LogicalSize` and `LogicalPosition` to convert it into IPC-compatible value that can be deserialized correctly on the Rust side into its equivalent struct.
