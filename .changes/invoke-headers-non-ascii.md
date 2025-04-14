---
"tauri": patch:bug
---

`invoke` will now properly throw when `options.headers` contains non-ascii characters instead of silently replacing them
