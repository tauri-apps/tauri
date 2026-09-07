---
'tauri-runtime-cef': 'patch:enhance'
---

Added `Cef::locale` and `Cef::accept_language_list`, exposing the matching `cef::Settings` fields so applications can pick the locale Chromium loads its own localized resources for and the languages sent in the `Accept-Language` header. The defaults are unchanged; note that the bundler currently ships only the `en-US` locale pak, so setting a locale whose pak is not bundled leaves Chromium unable to load its localized resources.
