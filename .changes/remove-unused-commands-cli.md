---
tauri-cli: 'minor:feat'
---

Reads `build > removeUnusedCommands` from the config file and pass in the environment variables on the build command to trigger the build scripts and macros to remove unused commands based on the capabilities you defined. For this to work on inlined plugins you must add a `#![plugin(<insert_plugin_name>)]` inside the `tauri::generate_handler![]` usage and the app manifest must be set.
