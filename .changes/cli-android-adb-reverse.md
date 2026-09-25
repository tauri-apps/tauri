---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix Android dev server port forwarding: `adb reverse --list` is now matched on the exact port (so `tcp:80` no longer matches `tcp:8080`), stale forwards on other connected devices are actually removed, and the forward verification gives up with a warning after a few attempts instead of retrying forever.
