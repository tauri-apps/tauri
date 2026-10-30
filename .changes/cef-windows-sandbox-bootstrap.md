---
"tauri-runtime-cef": minor:feat
"tauri-macros": minor:feat
"tauri-utils": minor:feat
"tauri-cli": minor:feat
"tauri-bundler": patch:bug
---

Windows CEF applications can now keep Chromium's process sandbox. `cef_entry_point` exports `RunWinMain` when applied to a library entry function, the runtime forwards the sandbox information object CEF's bootstrap supplies to `CefExecuteProcess` and `CefInitialize`, and the new `bundle > cef > windowsBootstrap` option makes the CLI ship `bootstrap.exe` as the application executable with the `cdylib` library as its sibling DLL. The bundler includes that DLL, signs it with the same certificate as the executable, and patches the bundle type into it. `tauri_runtime_cef::windows_bootstrap_active()` reports whether the current process was launched by the bootstrap host. Without the option, Windows still runs unsandboxed and the CLI warns at build time.

The Windows installers also skip optional CEF files that a distribution no longer ships, such as the ANGLE DLLs in CEF 154, instead of failing the bundle. Required CEF files are still copied and fail loudly when missing.
