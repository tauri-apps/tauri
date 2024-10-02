---
"@tauri-apps/cli": "patch:feat"
---

Implementd `toJSON` method on `PhysicalSize`, `PhysicalPosition`, `LogicalSize` and `LogicalPosition` to convert it into a valid JSON that can be deserialized correctly on the Rust side into its equivalent struct.
