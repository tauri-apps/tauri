---
"tauri": patch:bug
---

Properly deserialize Android plugin args with key starting with `is` (previously treated as a getter instead of a field name).
