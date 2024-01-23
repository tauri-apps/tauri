---
"tauri-build": patch:breaking
---

`CodegenContext::build` and `CodegenContext::try_build` have been removed, use `tauri_build::try_build(tauri_build::Attributes::new().codegen(codegen))` instead.
