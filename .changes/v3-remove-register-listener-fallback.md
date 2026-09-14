---
"@tauri-apps/api": major:breaking
---

`addPluginListener` no longer falls back to the camelCase `registerListener` command when `register_listener` fails. Plugins must expose the snake_case `register_listener` command.
