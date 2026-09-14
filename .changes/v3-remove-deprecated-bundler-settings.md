---
"tauri-bundler": major:breaking
"tauri-build": major:breaking
---

Removed the deprecated `AppImageSettings::bundle_xdg_open` and `WindowsSettings::icon_path` fields (the MSI installer icon is now always resolved from the `.ico` in `BundleSettings::icon`), and the deprecated `CodegenContext::config_path` method (use `Attributes::config_path` instead).
