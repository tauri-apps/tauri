---
"tauri-bundler": "minor:feat"
"tauri-cli": "patch:feat"
"@tauri-apps/cli": "patch:feat"
---

Added the `SRCTAURIDIR` NSIS define, holding the path of the directory that contains `tauri.conf.json`. It is defined before the installer hooks are included, so a hook can refer to files kept next to the config, for example `!define MUI_UNICON "${SRCTAURIDIR}\nsis\uninstall.ico"`.
