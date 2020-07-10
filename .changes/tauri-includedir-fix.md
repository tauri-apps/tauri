---
"tauri": patch
---

Properly ignore the `${distDir}/index.html` asset from the asset embbeding. Previously every asset with name matching `/(.+)index.html$/g` were ignored.
