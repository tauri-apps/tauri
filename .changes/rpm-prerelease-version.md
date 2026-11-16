---
"tauri-bundler": "patch:bug"
---

Fix `.rpm` bundling of a SemVer pre-release version such as `1.0.0-beta.1` failing with `invalid version` errors. The pre-release is now written in RPM's tilde form (`1.0.0~beta.1`), which sorts before the release as SemVer intends. The bundle's filename is unchanged.
