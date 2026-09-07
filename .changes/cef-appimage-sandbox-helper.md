---
'tauri-runtime-cef': 'patch:bug'
---

Fixed `LinuxSandboxPolicy::Auto` never taking effect inside an AppImage, which is the only situation it exists for. It probed for `chrome-sandbox` by existence, and Tauri's own AppImage bundler copies that file next to the main binary, so the probe always succeeded and the application still died with "No usable sandbox!" on a distribution that restricts unprivileged user namespaces. A helper inside an AppImage is now disregarded — the payload is mounted `nosuid`, so its setuid bit is inert — and anywhere else the helper (including one named by `CHROME_DEVEL_SANDBOX`) has to actually be a root-owned setuid binary executable by others, which is what Chromium requires before it will use one.
