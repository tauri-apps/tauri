---
"tauri-bundler": major:breaking
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

RPM bundles are compressed with zstd level 19 instead of gzip level 6 when `bundle > linux > rpm > compression` is not set, which matches rpm's own default. zstd payloads need rpm 4.14 or newer, so set the option to gzip to keep installing on older distributions.
