---
"tauri": "patch:bug"
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Rename the misspelled `choowHowToInstall` NSIS language string to `chooseHowToInstall`. A custom language file in `bundle.windows.nsis.customLanguageFiles` that only defines the old name keeps working for now, but users should migrate to the new name.
