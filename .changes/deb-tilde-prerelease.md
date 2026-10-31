---
"tauri-bundler": patch:enhance
---

Convert SemVer pre-release suffixes (e.g. `1.0.0-alpha`) to Debian's tilde syntax (`1.0.0~alpha`) in `.deb` bundles so they sort before the corresponding release.
