---
'tauri': 'patch:bug'
'tauri-utils': 'patch:bug'
'tauri-codegen': 'patch:bug'
---

On macOS, run the development binary from a minimal app bundle next to it (`target/<profile>/.tauri-dev/<binary>.app`) so it gets its own Dock icon and app switcher entry, instead of being registered under the app it was launched from (e.g. Terminal or VS Code).

Notes for development builds on macOS:

- The bundle uses the app's `identifier` as its `CFBundleIdentifier`, like the production bundle. Storage keyed by the bundle identifier (WebKit data such as `localStorage`, `NSUserDefaults`, notification permissions) is now shared with an installed build of the app, and data from earlier development runs is not migrated.
- LaunchServices registers the development bundle, so URL schemes declared in your `Info.plist` may open the development build instead of an installed one.
- `std::env::current_exe` returns the executable inside the development bundle. Use `tauri::utils::platform::current_exe` to get the cargo output path.
