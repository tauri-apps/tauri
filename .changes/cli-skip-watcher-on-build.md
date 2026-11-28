---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri build`, `tauri bundle`, `tauri android build`, `tauri ios build` and `tauri inspect` panicking with `Too many open files` when the OS refuses to create a new file watcher, for example when `fs.inotify.max_user_instances` is exhausted. These commands created an inotify watcher on `Cargo.toml` in `AppInterface::new` even though they never use it, and unwrapped the result. They now skip the watcher, and a failed watcher creation degrades to a warning instead of aborting the command.
