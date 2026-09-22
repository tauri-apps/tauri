---
"tauri": minor:feat
"tauri-utils": minor:feat
---

Added the `app > appDirectoriesOverride` config to override the directories returned by the `app_*_dir` path APIs, either with a single root directory or per directory. This is useful for portable apps that keep their data next to the executable and for tests.
