---
"tauri-bundler": patch:bug
---

Fix the macOS bundle failing on Xcode 27 with `actool: Bad file descriptor` when `bundle > icon` includes an Icon Composer `.icon` file, by running `actool` with a null stdin.
