---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix error messages that printed placeholders such as `{t}` literally instead of the value, e.g. "Could not find an Android device matching {t}".
