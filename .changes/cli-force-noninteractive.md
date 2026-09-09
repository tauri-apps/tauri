---
'tauri-cli': 'patch:bug'
---

When stdin is not a terminal, `tauri init` now automatically skips prompts, avoiding IO errors in CI and scripts. This eliminates the need to pass `--ci` explicitly in non-interactive environments.