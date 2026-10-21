---
"tauri-bundler": "patch:feat"
"tauri-cli": "patch:feat"
---

Support building macOS `.app` and `.dmg` bundles on Linux hosts when targeting Apple targets. Everything happens in-process, with no external tools to install: the `fstool` crate writes the HFS+ volume, the `udif` crate wraps it in a UDIF container, and `apple-codesign` produces the ad-hoc signature arm64 binaries need to launch.
