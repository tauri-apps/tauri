---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix MSI bundling failing with `LGHT0311` when the plain-text `licenseFile` contains characters outside Windows-1252 (for example Chinese, Cyrillic or Polish text), and `\`, `{` and `}` disappearing from the license shown in the installer. The text is now escaped when it is converted to RTF. A `licenseFile` ending in `.rtf` is also used as is again instead of being wrapped in a second RTF document.
