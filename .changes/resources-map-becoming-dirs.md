---
"tauri-utils": "patch:bug"
---

Fix `ResourcePaths` iterator returning an unexpected result for mapped resources, for example `"../resources/user.json": "resources/user.json"` generates this resource `resources/user.json/user.json` where it should generate just `resources/user.json`.
