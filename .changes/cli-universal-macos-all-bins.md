---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri build --target universal-apple-darwin` failing at the bundling step when the app has
more than one binary. Every binary returned by `get_binaries` is now merged with `lipo`, using the
same feature set cargo is built with, so config features (`build.features`) no longer make the
merged set smaller than the bundled one.
