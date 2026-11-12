---
"tauri": minor:feat
"tauri-runtime-wry": minor:feat
"tauri-build": minor:feat
"tauri-utils": patch:enhance
"@tauri-apps/api": minor:feat
---

On Android, each window is now an activity in a task of its own, which the system shows as a separate window: its own entry in Recents, its own split-screen pane and its own desktop window. Windows were previously stacked on the task of the window that opened them, so they behaved like several webviews in a single window. Windows now:

- Default to a new `TauriWindowActivity` when `activityName` is not set. Tauri generates it, and tauri-build declares it in `AndroidManifest.xml` (and adds the `androidx.activity` dependency it needs), so existing projects need no changes.
- Finish their activity and remove their task when closed (closing a window previously left its activity open).
- Are all finished when the app exits, not only the ones in the current task.

`supportsMultipleWindows` is now always `true` on Android.

Migration: a window created with `activityName` now opens in a task of its own, so activity embedding split rules no longer apply between it and the window that opened it. To keep showing two webviews side by side in one window, add the second webview to the window instead (`Window::add_child` with `WebviewBuilder::activity_name`, or `new Webview()` with `activityName`, which require the `unstable` feature).
