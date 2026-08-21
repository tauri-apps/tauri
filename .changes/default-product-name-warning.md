---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
"tauri-utils": patch:enhance
---

`tauri build` now warns when `productName` is still set to the default `tauri-app`, since it names the generated bundles and is written into install paths and metadata that are expected to be unique to your application. The config documentation for `productName` now lists what the field controls on each platform, and `identifier`'s documentation notes that the default value is rejected.
