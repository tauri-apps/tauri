---
"tauri-bundler": minor:feat
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
"tauri": minor:feat
"tauri-utils": minor:feat
---

Audio and video elements can now play `asset://` URLs on Linux. The `deb`, `rpm` and `pacman` bundles ship a GStreamer plugin in `/usr/lib/<productName>/gstreamer-1.0`, the `appimage` bundle includes it when `bundle > linux > appimage > bundleMediaFramework` is enabled, and `tauri dev` downloads it to the tools directory, where development binaries load it from. The plugin is downloaded at bundle time.

The plugin is only used when `app > security > assetProtocol > enable` is set. It reads the requested file directly, so media playback is not restricted by `assetProtocol > scope`.
