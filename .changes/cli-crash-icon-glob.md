---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
"tauri-bundler": "patch:bug"
---

Fix CLI crashing and failing to find a `.ico` file when `bundle > icon` option is using globs and doesn't have a string that ends with `.ico`.
