---
"tauri": minor:feat
"tauri-utils": minor:feat
---

Added the `app > appDirectoriesOverride` config to override the directories returned by the `app_*_dir` path APIs, either with a single root directory or per directory. This lets apps keep all of their data, including the data of Tauri itself and of plugins that use these APIs, in a single place of their choosing, such as next to the executable for portable apps. The override can also be set at runtime through `Context::config_mut`.
