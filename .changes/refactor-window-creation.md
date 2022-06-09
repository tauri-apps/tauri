---
"tauri": patch
---

**Breaking change:** Revert the window creation to be blocking in the main thread. This ensures the window is created before using other methods, but has an issue on Windows where the program deadlocks when creating a window in a Tauri command if it is not `async`. The documentation now states that commands must be `async` in other to prevent it until the issue is fixed in Webview2.
