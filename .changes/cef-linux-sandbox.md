---
'tauri-runtime-cef': 'patch:enhance'
---

Added `Cef::linux_sandbox` to control Chromium's Linux sandbox. The default (`LinuxSandboxPolicy::Auto`) keeps the sandbox enabled, and only passes `--no-sandbox` when the application runs from an AppImage on a system that offers no way to sandbox at all — AppImages cannot ship the setuid `chrome-sandbox` helper that the deb and rpm bundlers install, and distributions such as Ubuntu 23.10 and later restrict the unprivileged user namespaces Chromium falls back to, in which case Chromium would otherwise abort at startup with "No usable sandbox!". A warning naming the reason is logged whenever the sandbox is dropped. Use `LinuxSandboxPolicy::Required` to never drop it, or `LinuxSandboxPolicy::Disabled` to always run unsandboxed.
