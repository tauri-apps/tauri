---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix plugin scaffolding for names like `my-plugin`: the default Android package ID now uses a snake_case segment (`com.plugin.my_plugin`) and user-provided IDs are validated, iOS Xcode folders use the same kebab-case name as the project file, and the `plugin ios init`/`plugin android init` code snippets now match the generated bindings and the `setup(|app, api| ..)` API. `plugin init` into a non-empty directory no longer creates a `permissions` folder after skipping the template.
