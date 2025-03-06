---
tauri: 'minor:feat'
tauri-build: 'minor:feat'
tauri-codegen: 'minor:feat'
tauri-macros: 'minor:feat'
tauri-plugin: 'minor:feat'
tauri-utils: 'minor:feat'
---

Allow using an enviroment variable `REMOVE_UNUSED_COMMANDS` (you should avoid using it directly, use the command line arg `--remove-unused-commands` on the build command of tauri-cli instead) to trigger the build scripts and macros to remove unused commands based on the capabilities you defined, note this won't be accounting for dynamically added ACLs so make sure to check it when using this
