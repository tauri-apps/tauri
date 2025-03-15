---
tauri-cli: "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

The cli will now accept `--bundles updater` again. It's still no-op as it has been for all v2 versions. If you want to build updater artifacts, enable `createUpdaterArtifacts` in `tauri.conf.json`.
