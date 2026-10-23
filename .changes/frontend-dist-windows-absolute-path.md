---
"tauri-utils": patch:bug
---

Fix an absolute Windows path in `build > frontendDist` (e.g. `D:\dist`) being parsed as a URL with the drive letter as scheme instead of a directory, which prevented the assets from being embedded.
