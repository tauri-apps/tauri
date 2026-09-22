---
tauri: patch:perf
---

Skip redundant rgba buffer copies in icon conversions (`muda::Icon`/`tray_icon::Icon`), `Image::from_bytes`, and submenu icon construction.
