---
"tauri-bundler": "minor:feat"
---

Add `NSIS_HOOK_PREFINISHPAGE` NSIS installer hook, inserted right before the finish page is created, to allow `!undef`ing or redefining its `MUI_FINISHPAGE_*` defines (e.g. to remove the "Run" or "Create desktop shortcut" checkboxes).
