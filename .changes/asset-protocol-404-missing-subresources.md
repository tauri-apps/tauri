---
"tauri": patch:bug
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
"tauri-utils": patch:enhance
---

The `tauri://` asset protocol and the CLI's built-in dev server now return `404` (`text/plain`, naming the requested path) when a request that is not a navigation does not match any asset, instead of serving `index.html` with `200 text/html`. Navigations still resolve to the SPA `index.html` fallback so the frontend router can react to any URL, and that fallback now logs a warning. Requests are classified by `Sec-Fetch-Dest` where the webview sends it and by the `Accept` header otherwise; without either header the request counts as a navigation, so the fallback is preserved.
