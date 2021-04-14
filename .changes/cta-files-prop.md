---
"create-tauri-app": patch
---

CTA was missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
