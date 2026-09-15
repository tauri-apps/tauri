---
"tauri-bundler": "patch:bug"
---

The generated Linux desktop entry now carries an `Exec` field code when it declares `MimeType` associations: `%u` when deep-link schemes are configured, `%F` for file associations. Previously `Exec` had no field code, so `x-scheme-handler` activation launched the app without the URL and deep links were silently dropped.
