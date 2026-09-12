---
"tauri-macos-sign": patch:bug
---

Fixed importing a signing certificate failing with `MAC verification failed during PKCS12 import` for p12 files that use modern PBES2/AES encryption.
