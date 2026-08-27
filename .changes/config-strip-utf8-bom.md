---
"tauri-utils": "patch:bug"
---

Strip a leading UTF-8 BOM when reading the Tauri config file. `std::fs::read_to_string` keeps `U+FEFF`, and none of the JSON, JSON5 or TOML parsers accept it, so a BOM-prefixed config previously failed with an opaque `expected value at line 1 column 1` on a file that looks valid in an editor. This is easy to hit on Windows, where several common ways of writing the file (for instance PowerShell's `Set-Content -Encoding UTF8`) add a BOM.
