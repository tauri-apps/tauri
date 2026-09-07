---
'tauri-runtime-cef': 'patch:enhance'
---

Added `Cef::secret_storage` to pick which key Chromium uses to encrypt cookies and saved passwords at rest. The new default (`SecretStorage::Auto`) replaces the macOS keychain with a mock one in development builds, so ad-hoc-signed rebuilds stop asking for the "Chromium Safe Storage" keychain password on every launch, and always skips the D-Bus secret portal, libsecret and KWallet on Linux, where they can block startup on a keyring-unlock dialog or fail outright in a headless session. Windows keeps using DPAPI and is unaffected. Only cookies and saved passwords are involved — `localStorage` and IndexedDB are unencrypted either way — and the mock key is a public constant compiled into Chromium, so it offers no meaningful protection at rest; pass `SecretStorage::System` to always use the operating system secret store.
