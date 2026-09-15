---
"tauri": major:breaking
"tauri-macros": major:breaking
---

The internal channel plugin is now named `channel` (its data command is `plugin:channel|fetch` instead of `plugin:__TAURI_CHANNEL__|fetch`) and is a regular core plugin: `core:channel:default` is part of `core:default` and the command is checked against the ACL like any other. Data queued for that command is bound to the webview it was sent to, so a webview can no longer fetch another webview's payload, and it is dropped when the webview is destroyed. When a webview (including remote execution contexts) is not granted `core:channel:allow-fetch`, large `Channel` payloads and command responses still arrive, in order, through the slower inline path and a warning is logged once; add `core:channel:default` to such capabilities to keep the fast path.
