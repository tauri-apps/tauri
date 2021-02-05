---
"tauri": minor
---

The Tauri script is now injected with the webview `init` API, so it is available after page changes.
It will no longer be injected on `${distDir}/index.tauri.html`, but we will add a `${distDir}/__tauri.js` file to read it at app compile time.
You should add that to your `.gitignore` file.
