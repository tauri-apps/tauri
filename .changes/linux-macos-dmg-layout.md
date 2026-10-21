---
"tauri-bundler": patch:feat
---

Give DMGs built on Linux the Finder layout Tauri uses on macOS.

The Linux path now writes a real HFS+ volume (via the `fstool` crate) instead of an ISO 9660 image with Apple extensions. macOS's cd9660 driver does not expose Finder metadata, so the previous output ignored both the icon layout and the volume icon.

The staging directory gets a generated `.DS_Store` with the configured window size, icon size, and the app/Applications positions, plus a `.VolumeIcon.icns` copied from the app icon. The root folder's catalog record is patched afterwards to set `kHasCustomIcon`, which is what makes Finder use the icon.
