---
"tauri-bundler": "patch:bug"
---

Fix the Linux desktop file being named after `productName` while the window reports the main binary name as its application id (`StartupWMClass` on X11, Wayland `app_id`), so no environment could match windows back to the file when the two differ. On KDE Plasma Wayland this made every app whose `productName` contains spaces or capitals (e.g. `My App.desktop` vs app id `my-app`) fall back to the generic Wayland icon in the taskbar and Alt-Tab, since KWin resolves window icons by comparing the `app_id` to the desktop file name exactly. The desktop file is now named after the main binary name, matching the `Exec`, `Icon` and `StartupWMClass` already written into it; the user-visible `Name` is unaffected.
