---
"tauri-bundler": "patch:bug"
---

Fixed the AppImage bundler deleting the AppImages of other apps in the same Cargo workspace, and failing when two of them bundle at the same time. It now only cleans up and stages files for the app being bundled.
