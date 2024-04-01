---
'tauri-bundler': 'minor:feat'
'tauri-utils': 'minor:feat'
---

Added support for `provides`, `conflicts` and `replaces` (`obsoletes` for RPM) options for `bundler > deb` and `bundler > rpm` configs.

```json
{
  "bundler": {
    "deb": {
      "provides": ["my-package"],
      "conflicts": ["my-package"],
      "replaces": ["my-package"]
    },
    "rpm": {
      "provides": ["my-package"],
      "conflicts": ["my-package"],
      "obsoletes": ["my-package"]
    }
  }
}
```
