---
'tauri-bundler': 'patch:bug'
---

`output_ok` used to throw away the stderr it captured, so a failing bundler script surfaced as a bare "failed to run" with no diagnosis (#3055). The stderr is now part of the error, and when the DMG script dies because macOS denied the terminal permission to control Finder, the error says where to enable it: System Settings > Privacy & Security > Automation.
