---
'tauri-runtime-cef': 'patch:enhance'
---

Release builds now set `Settings::command_line_args_disabled`, so Chromium ignores switches passed on the shipped application's own command line. Without it anyone able to launch the app could also launch it with `--remote-debugging-port` and drive it over the DevTools protocol, or with `--disable-web-security`, `--proxy-server`, `--host-resolver-rules` or `--ssl-key-log-file`. Development builds are unaffected, switches configured through `Cef::command_line_arg` still apply, and Tauri's own CLI parsing and cold-start deep link handling read `std::env::args()`, which Chromium never touches. Deep links delivered to an already-running instance travel through Chromium's process singleton, so the runtime puts the deep link URL back on the cleared command line for that path. Applications that need users to pass Chromium switches can opt back in with `Cef::allow_chromium_command_line_args(true)`.
