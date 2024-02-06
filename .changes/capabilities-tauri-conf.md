---
"tauri-build": patch:breaking
"tauri-utils": patch:enhance
"tauri-codegen": patch:enhance
---

Allow defining capabilities inlined in the `tauri.conf.json > app > security > capabilities` configuration array and no longer automatically add capabilities to your app, you must link the identifier in that same capabilities array on the Tauri configuration file.
