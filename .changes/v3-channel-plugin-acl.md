---
"tauri": major:breaking
"tauri-macros": major:breaking
---

The internal channel plugin is now named `channel` (its data command is `plugin:channel|fetch` instead of `plugin:__TAURI_CHANNEL__|fetch`) and is a regular core plugin: `core:channel:default` is part of `core:default` and the command is checked against the ACL like any other. Apps whose capabilities do not include `core:default` must add `core:channel:default` to keep receiving large `Channel` payloads.
