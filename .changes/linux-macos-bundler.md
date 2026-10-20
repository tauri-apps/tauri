---
"tauri-bundler": "patch:feat"
"tauri-cli": "patch:feat"
---

Support building macOS `.app` and `.dmg` bundles on Linux hosts when targeting Apple targets. The Linux DMG path uses `genisoimage` and libdmg-hfsplus `dmg`, and `rcodesign` provides ad-hoc app signatures. External tools are resolved from the tools cache or `PATH` without runtime downloads.
