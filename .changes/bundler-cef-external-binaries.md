---
"tauri-bundler": "patch:bug"
---

Fix `sidecar()` failing to find external binaries in CEF apps installed from a deb or rpm. The deb and rpm bundlers moved only the main binary to `/usr/share/<product>/` and left external binaries in `/usr/bin`, but `sidecar()` looks beside the resolved executable. External binaries (and any extra binaries) now install beside the main binary, with a symlink left in `/usr/bin` for each one.
