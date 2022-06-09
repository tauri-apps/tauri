---
"tauri-bundler": patch
---

Only png files from tauri.conf.json > tauri.bundle.icon are used for app icons for linux targets.
Previously, all sizes from all source files (10 files using tauricon defaults) were used.
This also prevents unexpectedly mixed icons in cases where platform-specific icons are used.
