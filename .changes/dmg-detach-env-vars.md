---
"tauri-bundler": patch:feat
---

Allow customizing DMG detach retry behavior via environment variables `TAURI_DMG_DETACH_RETRIES` (default: 3) and `TAURI_DMG_DETACH_DELAY` (default: 1s base for exponential backoff). Fixes CI timeouts caused by `hdiutil detach` DiskArbitration expiration.
