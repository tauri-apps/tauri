---
"tauri-cli": minor:changes
"@tauri-apps/cli": minor:changes
---

The Android template now declares `MainActivity` with `android:launchMode="singleInstancePerTask"` instead of `singleTask`, so each window can be a new instance of it in a task of its own. Like `singleTask`, it keeps a single instance per task, so deep links and launcher taps still reach the running window through `onNewIntent`.

Migration: existing projects must update `src-tauri/gen/android/app/src/main/AndroidManifest.xml` to open more than one window without setting `activityName`. Change `android:launchMode="singleTask"` on `MainActivity` to `android:launchMode="singleInstancePerTask"`. Alternatively, create windows with `activityName` set to an activity of your own that extends `TauriActivity` and is declared with the `standard` or `singleInstancePerTask` launch mode. A `singleTask` activity can only have one instance, so opening a second window with it fails.
