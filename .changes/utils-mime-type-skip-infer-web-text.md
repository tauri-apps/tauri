---
"tauri-utils": patch:bug
---

`MimeType::parse` now uses the file extension first and only sniffs the content for unknown extensions, so a script that contains `%PDF` near its start is no longer served as `application/pdf`.
