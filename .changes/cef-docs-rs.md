---
"tauri-runtime-cef": patch:bug
---

Fix the docs.rs build: the documentation is now built with the `cef/dox` feature, which skips downloading the CEF binary distribution in the network-less docs.rs sandbox, for the Linux, Windows and macOS targets.
