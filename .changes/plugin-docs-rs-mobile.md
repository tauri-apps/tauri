---
"tauri-plugin": patch:bug
---

Skip the Android and iOS project setup in `Builder::build` and `Builder::try_build` when building on docs.rs, which mounts the sources read-only, so plugins with native mobile projects can build their documentation for mobile targets.
