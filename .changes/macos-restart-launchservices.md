---
'tauri': 'patch:bug'
---

On macOS, `process::restart` (and `AppHandle::restart`) now relaunches the app bundle through LaunchServices (`open -n`) so the new instance no longer inherits the exiting process's stdio and process group, which crashed it on its first print when the original stdout/stderr reader was gone (e.g. launched from a terminal that was closed). Arguments are still forwarded.
