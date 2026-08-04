---
"tauri-bundler": patch
---

Notarize and staple the signed DMG after signing it, mirroring the existing app-bundle notarization path. A build previously reported success while the downloaded DMG carried no notarization ticket, so Gatekeeper rejected it. The new path respects the `skip_stapling` opt-out and keeps the missing-team-id hard error, matching the app behavior.
