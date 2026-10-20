---
'tauri-build': 'patch:bug'
---

Don't watch the `capabilities` directory when it doesn't exist and the configuration lists capabilities in `app > security > capabilities`. Cargo treats a missing watched path as changed, so these apps re-ran the build script and recompiled on every build.
