---
"tauri": "patch:bug"
---

Apply `minWidth`, `minHieght`, `maxWidth` and `maxHeight` constraints separately, which fixes a long standing bug where these constraints were never applied unless width and height were constrained together.
