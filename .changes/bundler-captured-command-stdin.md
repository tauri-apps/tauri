---
"tauri-bundler": "patch:bug"
---

Run the commands whose output the bundler captures with an empty stdin instead of the inherited one. Under the Node.js CLI the inherited descriptor is close-on-exec, so `actool` started with no stdin at all and crashed on macOS (`-[__NSPlaceholderArray initWithObjects:count:]: attempt to insert nil object`), failing the bundle of any app with a `.icon` with `failed to run actool`.
