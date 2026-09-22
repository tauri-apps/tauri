---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix the `tauri capability new` command description, which said "Create a new permission file", and the `--skip-stapling` help text on `tauri build` and `tauri bundle`, whose first line described the opposite of what the flag does.
