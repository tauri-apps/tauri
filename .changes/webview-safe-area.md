---
'@tauri-apps/api': minor:feat
---

Add `safeArea.getSafeAreaInsets()` to read the current document's CSS safe-area insets in CSS pixels. The result reflects the webview engine's `env(safe-area-inset-*)` values, including immersive OpenHarmony pages with `viewport-fit=cover`.
