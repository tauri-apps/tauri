---
"tauri-utils": "patch:bug"
---

Fix a regression in tauri-utils 2.8.3 that made empty path an invalid resource target, e.g.

```json
{
  "bundle": {
    "resources": {
      "README.md": "",
    }
  }
}
```

(this means `README.md` -> `$RESOURCE/README.md`, note this is a confusing behavior, and will be changed in v3)
