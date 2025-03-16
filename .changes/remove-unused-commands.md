---
tauri: 'minor:feat'
tauri-build: 'minor:feat'
tauri-codegen: 'minor:feat'
tauri-macros: 'minor:feat'
tauri-plugin: 'minor:feat'
tauri-utils: 'minor:feat'
---

Added `build > removeUnusedCommands` to trigger the build scripts and macros to remove unused commands based on the capabilities you defined. Note this won't be accounting for dynamically added ACLs so make sure to check it when using this.
