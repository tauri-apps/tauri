---
"tauri": patch
---

Fix `fs.readDir` recursive option reading symlinked directories that are not allowed by the scope.
