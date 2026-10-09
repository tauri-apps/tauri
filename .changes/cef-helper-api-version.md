---
"tauri-runtime-cef": patch:bug
---

Fixed CEF child processes ignoring `Cef::cef_api_version`. Renderer, GPU and utility processes now declare the same CEF API version as the browser process, which passes it on each child's command line as `--tauri-cef-api-version=<n>`.
