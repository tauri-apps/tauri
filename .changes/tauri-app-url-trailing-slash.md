---
"tauri": patch:bug
---

On macOS, iOS and Linux the app is now loaded at `tauri://localhost/` instead of `tauri://localhost`, matching the `http://tauri.localhost/` URL used on Windows and Android. Third-party scripts that read `location.href`, such as Cloudflare Turnstile, failed on the URL with an empty path.
