---
"tauri-bundler": patch:enhance
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Infer macOS codesign identity from the `APPLE_CERTIFICATE` environment variable when provided, meaning the identity no longer needs to be provided when signing on CI using that option. If the imported certificate name does not match a provided signingIdentity configuration, an error is returned.
