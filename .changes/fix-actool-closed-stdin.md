---
"tauri-bundler": patch:bug
---

Fix the macOS bundle failing under the Node CLI with `failed to run actool` (`Bad file descriptor` or an `NSPlaceholderArray` exception, depending on the Xcode version) when `bundle > icon` includes an Icon Composer `.icon` file, by running `actool` with a null stdin.
