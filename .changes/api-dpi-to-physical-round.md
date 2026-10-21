---
"@tauri-apps/api": patch:bug
---

Round the result of `LogicalSize.toPhysical` and `LogicalPosition.toPhysical` to whole pixels. Fractional physical values were rejected when passed to APIs that take a `Size` or `Position`, such as `Window.setSize` and `Window.setPosition`.
