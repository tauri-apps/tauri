---
"tauri-build": patch:breaking
"tauri-utils": patch:enhance
"tauri-codegen": patch:enhance
---

Added a new configuration option `tauri.conf.json > app > security > capabilities` to reference existing capabilities and inline new ones. If it is empty, all capabilities are still included preserving the current behavior.
