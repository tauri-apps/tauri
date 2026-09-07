---
'tauri-runtime-cef': 'patch:enhance'
---

Release builds now set `Settings::command_line_args_disabled`, so Chromium ignores switches passed on the shipped application's own command line. Without it anyone able to launch the app could also launch it with `--remote-debugging-port` and drive it over the DevTools protocol, or with `--disable-web-security`, `--proxy-server`, `--host-resolver-rules` or `--ssl-key-log-file`. Development builds are unaffected, switches configured through `Cef::command_line_arg` still apply, and Tauri's own CLI and deep link argument handling is untouched. Applications that need users to pass Chromium switches can opt back in with `Cef::allow_chromium_command_line_args(true)`.
