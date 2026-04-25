---
"tauri-bundler": patch
---

Add missing `MULTIUSER_*` LangString translations to Korean NSIS language file (`MULTIUSER_TEXT_INSTALLMODE_TITLE`, `MULTIUSER_TEXT_INSTALLMODE_SUBTITLE`, `MULTIUSER_INNERTEXT_INSTALLMODE_TOP`, `MULTIUSER_INNERTEXT_INSTALLMODE_ALLUSERS`, `MULTIUSER_INNERTEXT_INSTALLMODE_CURRENTUSER`). Previously these fell back to English when a Korean installer used `installMode: both`.
