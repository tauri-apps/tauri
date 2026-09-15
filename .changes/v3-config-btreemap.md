---
"tauri-utils": major:breaking
"tauri": major:breaking
"tauri-runtime-cef": major:breaking
---

`Csp::DirectiveMap`, `HeaderSource::Map` and `PluginConfig` now hold a `BTreeMap` instead of a `HashMap`, so serialization and the generated context are deterministic without the custom `Serialize` and `ToTokens` workarounds. The `From` conversions between `Csp` and its directive map now use `BTreeMap<String, CspDirectiveSources>`.
