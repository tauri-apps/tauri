---
"tauri-runtime-cef": patch:bug
---

Fixed the app reopening hidden copies of its previous run's windows, which kept the process alive after every app window was closed. After a crash or a kill, Chromium's session restore reopened the last session's tabs in Chrome browser windows the runtime does not own. The runtime now sets the `session.restore_on_startup` profile preference to open the New Tab page, which restores nothing. `Cef::profile_preference_value` can still override it.
