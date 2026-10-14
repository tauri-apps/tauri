---
"tauri-macos-sign": patch:bug
---

Fixed importing a signing certificate failing with `MAC verification failed during PKCS12 import` for p12 files that use modern PBES2/AES encryption. The certificate format is now detected and passed to `security import` explicitly (PKCS#12 or a PEM bundle), and import failures explain the expected certificate format and password.
