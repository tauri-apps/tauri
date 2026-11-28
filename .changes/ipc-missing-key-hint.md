---
"tauri": "patch:enhance"
---

When a command is missing a required argument but the payload has a key that only differs in case or underscores, such as `on_raw` for `onRaw`, the error now names that key. Other missing keys keep the previous message.
