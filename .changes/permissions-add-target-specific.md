---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

`permission add` and `add` commands now check if the plugin is known and if it is either desktop or mobile only
we add the permission to a target-specific capability.
