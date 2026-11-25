---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Do not require an iOS simulator runtime for `tauri ios build` and `tauri ios dev` when the target is a device. The CLI now checks the iPhoneOS SDK for device targets. It still checks the simulator runtime for simulator targets.
