---
"tauri": patch
---

Replace `\` with `\\` in css assets that are lazy loaded. Since these are injected in a template literal, backslashes must be escaped. Backslashes are sometimes used for octal sequences in CSS.
