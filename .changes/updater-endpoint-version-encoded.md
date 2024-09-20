---
'tauri': 'patch:bug'
---

Encode `+` when making updater requests which can be cause incorrectly interpolating the endpoint when using `{{current_version}}` in the endpoint where the current version contains a build number, for example `1.8.0+1`.
