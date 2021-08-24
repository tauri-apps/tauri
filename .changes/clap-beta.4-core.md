---
"tauri": patch
---

Change `Error::ParseCliArguments(clap::Error)` to `Error::ParseCliArguments(String)` because `clap::Error` is not `Send`.
