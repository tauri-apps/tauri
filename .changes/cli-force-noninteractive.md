---
'tauri-cli': 'patch:enhance'
'@tauri-apps/cli': 'patch:enhance'
---

When stdin is not a terminal, `tauri init` now automatically skips prompts, avoiding IO errors in CI and scripts. This eliminates the need to pass `--ci` explicitly in non-interactive environments.
