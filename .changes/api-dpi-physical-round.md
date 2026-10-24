---
"@tauri-apps/api": minor:feat
---

Added `PhysicalSize.round`, `PhysicalPosition.round`, `LogicalSize.round` and `LogicalPosition.round`, which round to whole pixels the same way Rust does, for example `new LogicalSize(301, 201).toPhysical(1.5).round()`. APIs that take a `Size` or `Position`, such as `Window.setSize` and `Window.setPosition`, only accept whole physical pixels.
