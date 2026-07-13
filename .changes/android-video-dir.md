---
tauri: minor:breaking
---

On Android, `$VIDEO` and `video_dir()` now resolve to the app-specific Movies directory instead of external cache storage.

**Migration:** Files previously written to the old location (`.../cache`) will not be discovered at the new location (`.../files/Movies`). Migrate existing files or update path assumptions accordingly.
