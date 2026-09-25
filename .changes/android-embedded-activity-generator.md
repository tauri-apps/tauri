---
"tauri-build": minor:feat
---

Enable Android Activity embedding when `tauri.conf.json > bundle > android > activityEmbedding > enabled` is true: generate the Gradle dependencies, manifest entries and split rules from `splitRules`, plus a default `TauriActivity` subclass for each secondary activity not defined by the app. Generated files and manifest entries are removed again when the feature is disabled.
