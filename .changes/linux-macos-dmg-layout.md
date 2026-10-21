---
"tauri-bundler": patch:feat
---

Give DMGs built on Linux the Finder layout Tauri uses on macOS. The staging directory now contains a generated `.DS_Store` with the configured window size, icon size, and the app/Applications icon positions, plus a `.VolumeIcon.icns` copied from the app icon, so the mounted image no longer opens as an unstyled window.

`genisoimage` only writes Apple's `AA` Finder-info entry for regular files, so the volume root is patched afterwards to carry the "has custom icon" flag. Without it Finder ignores `.VolumeIcon.icns` and the image keeps the generic disk icon.
