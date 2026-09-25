---
"tauri-bundler": "patch:bug"
---

Recognize `.deb` and `.rpm` as self contained updater artifacts (the updater plugin installs them directly): the "no updater-enabled targets were built" warning is no longer printed when only a `.rpm` bundle is produced, and it now lists both targets. Setting `createUpdaterArtifacts` to `"v1Compatible"` no longer fails with "Unable to find a bundled project for the updater" when only a `.deb` bundle is built; the legacy updater never supported `.deb`, so the bundler now warns that no v1 compatible artifact was created instead.
