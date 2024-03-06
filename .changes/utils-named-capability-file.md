---
"tauri-utils": major:breaking
---

Changed `CapabiltyFile::List` enum variant to be a tuple-struct and added `CapabiltyFile::NamedList`. This allows more flexibility when parsing capabilties from JSON files. 