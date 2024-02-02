---
'tauri-utils': 'patch:breaking'
---

Changed `error` field in `ConfigError::FormatToml` to be boxed `Box<toml::de::Error>` to reduce the enum `ConfigError` size in memory.
