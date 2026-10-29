---
"tauri-runtime-cef": minor:feat
"tauri-macros": minor:feat
"tauri-utils": minor:feat
"tauri-cli": minor:feat
"tauri-bundler": patch:bug
---

Windows CEF applications can now keep Chromium's process sandbox. `cef_entry_point` exports `RunWinMain` when applied to a library entry function, the runtime forwards the sandbox information object CEF's bootstrap supplies to `CefExecuteProcess` and `CefInitialize`, and the new `bundle > cef > windowsBootstrap` option makes the CLI ship `bootstrap.exe` as the application executable with the `cdylib` library as its sibling DLL. The bundler includes that DLL, signs it with the same certificate as the executable, and patches the bundle type into it. Without the option, Windows still runs unsandboxed and the CLI warns at build time.
