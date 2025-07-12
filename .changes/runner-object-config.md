---
'tauri-cli': 'minor:feat'
'tauri-utils': 'minor:feat'
---

Allow runner configuration to be an object with cmd, cwd, and args properties. The runner can now be configured as `{ "cmd": "my_runner", "cwd": "/path", "args": ["--quiet"] }` while maintaining backwards compatibility with the existing string format.
