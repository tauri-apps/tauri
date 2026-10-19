---
"tauri-bundler": "patch:bug"
---

Fix NSIS installers registering file associations with an unquoted executable path, which broke opening associated files when the install directory contained spaces (common for per-user installs under a profile whose name has a space).
