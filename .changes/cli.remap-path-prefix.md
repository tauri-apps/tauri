---
"cli.rs": "patch"
---

Configure the rust compiler to truncate absolute paths in panic messages and debug symbols when building in release mode. This prevents a possible leak of PII through absolute paths.