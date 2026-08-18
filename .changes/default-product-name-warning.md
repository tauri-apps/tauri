---
"tauri-cli": patch:enhance
"tauri-utils": patch:enhance
---

`tauri build` now warns when `productName` is still set to the default `tauri-app`, since it is used to derive the default Windows installer (WiX) upgrade code, which must be unique across applications. The config documentation for `productName` and `identifier` was updated accordingly.
