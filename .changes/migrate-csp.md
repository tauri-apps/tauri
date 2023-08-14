---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Update migrate command to update the configuration CSP to include `ipc:` on the `connect-src` directive, needed by the new IPC using custom protocols.
