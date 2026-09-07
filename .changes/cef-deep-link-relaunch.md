---
'tauri-runtime-cef': 'patch:bug'
---

Fixed deep links being dropped in release builds when the application was already running. `Settings::command_line_args_disabled` makes CEF clear the second process's command line before Chromium's process singleton relays it, so `on_already_running_app_relaunch` received no arguments and the `myapp://...` URL was lost — clicking a deep link while the app was open did nothing on Windows and Linux. The browser process now puts any argument whose scheme matches a configured deep link scheme back onto the command line; every other argument stays dropped, which is what the lockdown is for.
