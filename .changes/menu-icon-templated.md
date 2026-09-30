---
"tauri": "minor:feat"
---

Add `IconMenuItem::set_icon_templated` and `Submenu::set_icon_templated`, to set a menu item's icon and draw it as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image on macOS, so the system recolours it to match the menu the way the built-in items are. Also add `IconMenuItem::icon_is_template` and `Submenu::icon_is_template` to read that state back.
