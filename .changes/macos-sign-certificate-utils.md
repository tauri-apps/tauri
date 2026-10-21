---
'tauri-macos-sign': 'minor:feat'
---

Added utilities to generate private keys and certificate signing requests, create and read PKCS#12 files (`certificate` module), export signing identities from the keychain (`export_identity`, `find_identities`), import certificates into the login keychain (`import_into_login_keychain`), read provisioning profiles without the `security` CLI (`ProvisioningProfileInfo`) and load App Store Connect API keys from their contents (`ApiKey::from_content`).
