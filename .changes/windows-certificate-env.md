---
'tauri-bundler': 'minor:feat'
'tauri-cli': 'minor:feat'
'@tauri-apps/cli': 'minor:feat'
---

On Windows, the bundler now imports the base64 encoded `.pfx` certificate from the `WINDOWS_CERTIFICATE` environment variable (protected by `WINDOWS_CERTIFICATE_PASSWORD`) into the current user's certificate store and signs with it, so `certificateThumbprint` is no longer required and CI/CD platforms no longer need a separate step to import the certificate.
