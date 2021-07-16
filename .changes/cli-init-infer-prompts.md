---
"cli.rs": patch
---

Infer `app name` and `window title` from `package.json > productName` or `package.json > name`.
Infer `distDir` and `devPath` by reading the package.json and trying to determine the UI framework (Vue.js, Angular, React, Svelte and some UI frameworks).
