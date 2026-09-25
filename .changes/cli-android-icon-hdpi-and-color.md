---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri icon` now generates 72x72 Android `hdpi` launcher icons (previously 49x49) and writes the Android launcher background color in `#RRGGBB`/`#AARRGGBB` notation instead of the raw CSS color string, which Android rejected or misread. Invalid SVG sources and `--png 0` now return an error instead of panicking.
