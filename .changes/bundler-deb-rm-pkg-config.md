---
'cli.js': patch
'cli.rs': patch
---

No longer adds the `pkg-config` dependency to `.deb` packages when the `systemTray` is used.
This only works with recent versions of `libappindicator-sys` (including https://github.com/tauri-apps/libappindicator-rs/pull/38),
so a `cargo update` may be necessary if you create `.deb` bundles and use the tray feature.
