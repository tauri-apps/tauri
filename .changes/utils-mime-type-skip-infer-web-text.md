---
"tauri-utils": patch:bug
---

Don't sniff the content of `.js`, `.mjs`, `.css`, `.html` and `.json` files when guessing a MIME type, so a script that contains `%PDF` near its start is no longer served as `application/pdf`.
