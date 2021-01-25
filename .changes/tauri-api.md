---
"tauri.js": minor
---

The Tauri API interface is now shipped with the `@tauri-apps/api` package instead of the deprecated `tauri` package.
To use the new API package, delete the old `tauri` from your `package.json` and install the new package:
`$ yarn remove tauri && yarn add @tauri-apps/api` or `$ npm uninstall tauri && npm install @tauri-apps/api`.
And change all `import { someApi } from 'tauri/api` to `import { someApi } from '@tauri-apps/api'`.
