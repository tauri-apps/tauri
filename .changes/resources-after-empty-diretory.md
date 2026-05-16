---
"tauri-utils": "patch:bug"
---

Fix a regression in tauri-utils 2.8.3 that made an empty directory makes it skip all the following entries, e.g.

```json
{
  "bundle": {
    "resources": [
      "empty-directory",
      "README.md"
    ]
  }
}
```

if `empty-directory` is empty, the `README.md` will not be copied to the resource directory (skipped)
