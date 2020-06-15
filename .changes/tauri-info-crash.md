---
"tauri.js": patch
---

Fixes the case when `tauri info` is run and a project has not yet created a `Cargo.lock` closing [#610](https://github.com/tauri-apps/tauri/issues/610).
