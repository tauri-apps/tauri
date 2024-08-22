---
"tauri-cli": patch:breaking
"@tauri-apps/cli": patch:breaking
---

The `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD` and `IOS_MOBILE_PROVISION` environment variables are now read by the `ios build` command instead of `ios init`.
