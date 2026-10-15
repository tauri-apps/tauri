---
"tauri-runtime-cef": patch:enhance
---

The build script now passes the CEF binary distribution `cef-dll-sys` resolved on to the build scripts of dependents, as `DEP_TAURI_RUNTIME_CEF_CEF_DIR`.
