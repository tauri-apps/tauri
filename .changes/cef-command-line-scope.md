---
'tauri-runtime-cef': 'patch:bug'
---

Command line arguments configured through `Cef::command_line_arg` and `Cef::command_line_args` are now applied to the browser process only. CEF documents that modifying the command line of a non-browser process "may result in undefined behavior including crashes", and Chromium already forwards to each renderer, GPU and utility process the switches it needs. The runtime's own internal switches (`--no-first-run`, and `ozone-platform=x11` on Linux) keep being applied to every process type.
