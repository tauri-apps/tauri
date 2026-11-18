---
"tauri-utils": patch:bug
---

`MimeType::parse` now uses the file extension for text types like `.js`, `.css` and `.svg` instead of sniffing the content, so a script that contains `%PDF` near its start is no longer served as `application/pdf`.
