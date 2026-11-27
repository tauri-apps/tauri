---
'tauri-cli': 'minor:feat'
'@tauri-apps/cli': 'minor:feat'
---

Added commands to set up code signing for distribution:

- `tauri signer android generate-keystore` creates an upload keystore and configures the Android project to sign release builds with it, and `tauri signer android configure-keystore` does the same with an existing keystore.
- `tauri signer apple create-certificate` creates a signing certificate (and optionally an iOS provisioning profile) with the App Store Connect API.
- `tauri signer apple csr` and `tauri signer apple import-certificate` create a certificate signing request and turn the certificate downloaded from the Apple Developer portal into a `.p12` file, without Keychain Access.
- `tauri signer apple export-certificate` exports a signing certificate from the macOS keychain to a `.p12` file.
- `tauri signer apple export-profile` and `tauri signer apple export-api-key` encode a provisioning profile and an App Store Connect API key for CI/CD platforms.
- `tauri signer windows import-certificate` converts a certificate and its private key to a `.pfx` file, prints its thumbprint and configures `bundle > windows > certificateThumbprint`.

The commands output the environment variables to set in CI/CD platforms, and can store them as GitHub Actions secrets with `--github-secrets` or write them to a dotenv file with `--env-file`.
