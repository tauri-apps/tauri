---
'tauri-cli': 'patch'
'@tauri-apps/cli': 'patch'
---

The `tauri plugin` subcommand is receving a couple of consitency and quality of life improvements:

- Renamed `tauri plugin android/ios add` command to `tauri plugin android/ios init` to match the `tauri plugin init` command.
- Removed the `-n/--name` argument from the `tauri plugin init`, `tauri plugin android/ios init`, and is now parsed from the first positional argument.
- Added `tauri plugin new` to create a plugin in a new directory.
- Changed `tauri plugin init` to initalize a plugin in an existing directory (defaults to current directory) instead of creating a new one.
- Changed `tauri plugin init` to NOT generate mobile projects by default, you can opt-in to generate them using `--android` and `--ios` flags or `--mobile` flag or initalize them later using `tauri plugin android/ios init`.
