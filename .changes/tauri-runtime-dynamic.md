---
"tauri-runtime": minor:feat
---

Added the `tauri_runtime::dynamic` module with `DynRuntime`, a type-erased `Runtime` that wraps any concrete runtime selected at build time through `DynRuntimeInitAttrs`, along with `DynWebview`, `DynWindowOpener` and `DynWebviewAttribute` wrappers that can be downcast to the runtime's types.
