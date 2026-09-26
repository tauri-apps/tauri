---
"tauri-bundler": "minor:feat"
"tauri-utils": "minor:feat"
"tauri-cli": "minor:feat"
"@tauri-apps/cli": "minor:feat"
---

Added an experimental AppImage bundler based on sharun and uruntime, enabled with the `bundle > linux > appimage > useNewFormat` config option or the `TAURI_BUNDLER_NEW_APPIMAGE_FORMAT` environment variable. The resulting AppImage carries its own dependencies, so it runs on distributions older than the build host, does not depend on the host libc and supports Wayland without forcing the use of XWayland. Update information for AppImageUpdate can be set with the new `bundle > linux > appimage > updateInformation` option. The existing linuxdeploy based bundler still is the default and is now deprecated.
