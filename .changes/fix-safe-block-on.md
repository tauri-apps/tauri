---
"tauri": patch
---

Fixes `Command::output` and `Command::status` deadlock when running on async commands.
