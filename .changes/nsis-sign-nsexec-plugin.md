---
"tauri-bundler": "patch:bug"
---

Sign the stock NSIS `nsExec.dll` plugin along with the other stock plugins when a signing command or certificate is configured, so installers whose hooks call `nsExec::Exec*` don't load an unsigned DLL.
