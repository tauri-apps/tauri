---
'tauri': 'patch:bug'
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

On iOS, mark the Swift `@_cdecl` entry points that Rust links against as `public` (in the iOS API and in the plugin templates), so release builds made with Xcode 27 keep them global and link.
