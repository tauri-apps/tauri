---
"tauri": patch
---

Expose mutable getters for the rest of the public `Context` getters.
* `pub fn assets_mut(&mut self) -> &mut Arc<A>`
* `pub fn default_window_icon_mut(&mut self) -> &mut Option<Vec<u8>>`
* `pub fn system_tray_icon_mut(&mut self) -> &mut Option<Icon>`
* `pub fn package_info_mut(&mut self) -> &mut tauri::api::PackageInfo`
