---
"tauri": minor:feat
"@tauri-apps/api": minor:feat
---

Added iOS support for `Window::set_badge_count`, `WebviewWindow::set_badge_count` and the JavaScript `Window#setBadgeCount`, which sets the app icon badge. They were only available on desktop: the methods were desktop-only and the window plugin only registered the `set_badge_count` command there. They are now available on mobile too (a no-op on Android).
