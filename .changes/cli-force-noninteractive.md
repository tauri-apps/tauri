---
'tauri-cli': 'patch:bug'
---

`tauri init --force` now implies `--ci`, skipping all interactive prompts. This makes it safe for non-interactive environments like CI.