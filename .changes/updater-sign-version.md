---
"tauri-cli": "minor:feat"
"@tauri-apps/cli": "minor:feat"
---

Record the app version in the trusted comment of updater signatures, so a signed artifact is bound to the version it was released as.

An update endpoint response is not signed, and the signature only covers the downloaded artifact, so the announced `version` on its own does not prove which release the `url` and `signature` point at. minisign covers the trusted comment with its global signature, which lets the updater plugin compare the two and reject a response that pairs a version number with a different release. Enable `requireSignedVersion` in the updater plugin configuration to enforce this.

`tauri build` fills the version in automatically, and `tauri plugin add updater` now enables `requireSignedVersion` for the project it is adding the plugin to. `tauri signer sign` gains an `--app-version` flag for signing updater artifacts by hand, and warns when it is omitted.
