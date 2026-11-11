---
"tauri": patch:bug
---

On macOS, `process::restart` (and `AppHandle::restart`) no longer brings the relaunched app to the foreground, so it doesn't take focus from the app the user is working in.
