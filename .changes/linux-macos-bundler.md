---
"tauri-bundler": "patch:feat"
"tauri-cli": "patch:feat"
---

Support building macOS `.app` and `.dmg` bundles on Linux hosts when targeting Apple targets. The Linux DMG path writes a real HFS+ volume with `mkfs.hfsplus` and libdmg-hfsplus `hfsplus`, then converts it to UDIF with `dmg`, and `rcodesign` provides ad-hoc app signatures. External tools are resolved from the tools cache or `PATH` without runtime downloads.
