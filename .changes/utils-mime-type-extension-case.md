---
"tauri-utils": patch:bug
---

Match file extensions case-insensitively when guessing a MIME type, so files such as `app.JS` or `Logo.SVG` no longer get the fallback type (`text/html` for the `tauri://` and `asset://` protocols).
