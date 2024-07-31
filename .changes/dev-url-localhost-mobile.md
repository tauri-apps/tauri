---
"tauri-cli": patch:breaking
"@tauri-apps/cli": patch:breaking
---

`ios dev` and `android dev` now uses localhost for the development server unless running on an iOS device,
which still requires connecting to the public network address. To conditionally check this on your frontend
framework's configuration you can check for the existence of the `TAURI_DEV_PUBLIC_NETWORK_HOST_REQUIRED`
environment variable instead of checking if the target is iOS or Android (previous recommendation).
