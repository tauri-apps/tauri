---
'tauri': 'major:breaking'
'tauri-utils': 'major:breaking'
---

Restructured Tauri config per [RFC#5](https://github.com/tauri-apps/rfcs/blob/f3e82a6b0c5390401e855850d47dc7b7d9afd684/texts/0005-tauri-config-restructure.md):

- Moved `package.product_name`, `package.version` and `tauri.bundle.identifier` fields to the top-level.
- Removed `package` object.
- Renamed `tauri` object to `app`.
- Moved `tauri.bundle` object to the top-level.
- Renamed `build.distDir` field to `frontendDist`.
- Renamed `build.devPath` field to `devUrl` and will no longer accepts paths, it will only accept URLs.
- Moved `tauri.pattern` to `app.security.pattern`.
- Removed `tauri.bundle.updater` object, and its fields have been moved to the updater plugin under `plugins.updater` object.
- Moved `build.withGlobalTauri` to `app.withGlobalTauri`.
- Moved `tauri.bundle.dmg` object to `bundle.macOS.dmg`.
- Moved `tauri.bundle.deb` object to `bundle.linux.deb`.
- Moved `tauri.bundle.appimage` object to `bundle.linux.appimage`.
- Removed all license fields from each bundle configuration object and instead added `bundle.license` and `bundle.licenseFile`.
