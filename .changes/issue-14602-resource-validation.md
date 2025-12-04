---
"tauri-utils": "patch:bug"
"tauri-build": "patch:bug"
---

Skip resource path validation when `TAURI_SKIP_RESOURCE_CHECK=true` is set. This is useful for running `cargo check` or `cargo clippy` in CI/CD pipelines where external binaries might not be present.
