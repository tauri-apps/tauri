---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

`tauri add` no longer installs the npm package `tauri-plugin-<name>-api` for community plugins. npm and crates.io are separate namespaces, so that package may be unowned or squatted; the CLI now asks you to install the plugin's JavaScript bindings yourself. `tauri remove` only removes the plugin's JS package when `package.json` lists it, and a failure there no longer skips the capability cleanup.
