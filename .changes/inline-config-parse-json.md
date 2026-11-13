---
"tauri-utils": patch:perf
---

Mark `config::parse::parse_json` and `parse_json5` `#[inline]` so crates that don't call them no longer generate code for the `Config` deserializer.
