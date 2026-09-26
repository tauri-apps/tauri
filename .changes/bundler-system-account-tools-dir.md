---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix NSIS and MSI bundling failing with `Unable to start child process, error 0x2` when running as the Windows `SYSTEM` account, for example on a CI runner installed as a service. When the cache directory is inside `C:\Windows\System32`, where WOW64 file system redirection hides the downloaded tools from the 32-bit NSIS and WiX executables, the bundler now stores them in the project output directory instead.
