---
"tauri-bundler": "patch:bug"
---

Fix NSIS installer and uninstaller reading their command-line flags (`/P`, `/NS`, `/UPDATE`, `/R`, `/ARGS`, `/DELETEAPPDATA`) from their own path when started with an unquoted forward-slash path, which could e.g. turn on passive mode and skip the licence page.
