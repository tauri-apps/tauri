---
"tauri-macros": patch:enhance
"tauri-codegen": patch:enhance
---

The `generate_context` proc macro now accepts a `capabilities` attribute where the value is an array of file paths that can be [conditionally compiled](https://doc.rust-lang.org/reference/conditional-compilation.html). These capabilities are added to the application along the capabilities defined in the Tauri configuration file.
