---
'tauri-runtime-cef': 'minor:enhance'
---

Add `Cef::child_process_command_line_arg` and `Cef::child_process_command_line_args`, which append the given switches to every child process's command line through CEF's `OnBeforeChildProcessLaunch`. This is the supported way to give the GPU and renderer processes switches Chromium does not propagate itself, such as `--use-angle` or `--enable-features` entries an application needs for WebGPU to find a hardware adapter instead of falling back to SwiftShader.
