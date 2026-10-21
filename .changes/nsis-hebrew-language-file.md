---
"tauri-bundler": patch:bug
---

Include the Hebrew language file in the NSIS installer. `get_lang_data` had no arm for it, so `MUI_LANGUAGE "Hebrew"` loaded the NSIS language table but none of Tauri's own strings were ever set and Hebrew installers showed blank text.
