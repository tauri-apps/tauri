---
tauri-macos-sign: patch:deps
---

**Breaking Change:** Temporarily inlined the code we need from apple-codesign-rs until its next release to unblock important dependency updates. Technically this is a breaking change to the `CertificateProfile` enum re-export and the `Error::FailedToCreateSelfSignedCertificate` enum variant.
