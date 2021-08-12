---
"tauri": patch
---

Adds a `PathResolver` struct to simplify the usage of the `tauri::api::path::{app_dir, resource_dir}` APIs, accessible through the `App` and `AppHandle` `path_resolver` methods.
