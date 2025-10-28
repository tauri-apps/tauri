---
"tauri": patch:bug
"tauri-macros": patch:bug
---

Fix iOS deadlock when running on the simulator from Xcode by properly piping stdout/stderr messages through the Xcode console and OSLog.
