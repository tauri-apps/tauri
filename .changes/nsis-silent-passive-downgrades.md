---
"tauri": "patch:bug"
"tauri-bundler": "patch:bug"
---

Fix NSIS installers not blocking downgrades in silent (`/S`), passive (`/P`) and update (`/UPDATE`) modes when `bundle.windows.allowDowngrades` is `false`.
