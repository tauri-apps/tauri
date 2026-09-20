---
"tauri-bundler": "patch:bug"
---

Fix the bundler warning about missing updater-enabled targets being printed when only a `.rpm` bundle is produced. Both `.deb` and `.rpm` are self contained updater artifacts (the updater plugin installs them directly) and are now recognized as such, and the warning lists them.
