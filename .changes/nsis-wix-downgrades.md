---
"tauri-bundler": "patch:bug"
---

Fix NSIS installers not blocking downgrades over a newer installation made by the MSI installer in silent (`/S`), passive (`/P`) and update (`/UPDATE`) modes when `bundle.windows.allowDowngrades` is `false`.
