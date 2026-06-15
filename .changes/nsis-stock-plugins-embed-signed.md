---
"tauri-bundler": "patch:bug"
---

Fix NSIS stock plugins (`NSISdl.dll`, `StartMenu.dll`, `System.dll`, `nsDialogs.dll`) being embedded in the final installer as unsigned despite the signing step succeeding. The signed local copies under `<output>/Plugins/x86-unicode/` were not on makensis' plugin search path, so makensis fell back to the unsigned DLLs from the NSIS toolset directory. The fix adds `!addplugindir` for the signed plugin directory before any plugin command is parsed in the script.
