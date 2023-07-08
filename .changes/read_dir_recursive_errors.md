---
'tauri': patch:bug
---

Fix `tauri::api::read_dir` and the JS API `fs.readDir` failing if one of the children was not accessible when `recursive` option was `true`.
