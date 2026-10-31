---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
"@tauri-apps/api": minor:feat
---

Support the multiwebview API on Android (`Window::add_child` and `new Webview()`, behind the `unstable` feature):

- A window's first webview lives in the window's activity.
- Each other webview lives in an activity of its own, set with the new `WebviewBuilder::activity_name` or the `activityName` webview option and started in the window's task. [Activity embedding](https://developer.android.com/develop/ui/views/layout/activity-embedding) rules can show it next to the window's webview on large screens; position and size are ignored.
- `Webview::close` finishes that activity. When the activity is closed by the user (for instance with the back button), the webview is destroyed and the new `WebviewEvent::Destroyed` event (`tauri://destroyed`) is emitted.
