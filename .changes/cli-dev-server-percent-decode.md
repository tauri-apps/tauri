---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix the builtin dev server serving `index.html` instead of files whose names contain spaces or non-ASCII characters, such as `my font.woff2`. The request path is now percent-decoded before looking up the file, as the `tauri://` protocol does in production builds.
