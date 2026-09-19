---
"tauri": patch:bug
---

Panic at build time when two plugins share the same name in `Builder::plugin()`, instead of silently replacing one and causing cryptic errors later.
