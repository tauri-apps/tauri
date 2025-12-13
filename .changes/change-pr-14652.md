---
"tauri-cli": patch:bug
---

Adjust `Pbxproj::set_build_settings` to ensure we are not writing lines outside
configuration blocks.
