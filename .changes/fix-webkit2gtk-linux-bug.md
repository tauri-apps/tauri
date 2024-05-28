---
'tauri-bundler': 'patch:bug'
---

Fix the `libwebkit2gtk-4.0` bug in Linux by embedding them into the `deb` package. This increases the size of package, but is necessary to make `deb` packages work on Ubuntu 24 and Debian 13 and higher.
