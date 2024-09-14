---
"tauri": patch:bug
---

Fixes the restart() function not being compatible with the v2 binary name change.
Additionally, do not panic if we somehow failed to restart, and only exit instead.
