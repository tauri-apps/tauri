---
"tauri-cli": "patch:enhance"
"@tauri-apps/cli": "patch:enhance"
---

Enhance `tauri migrate` to also migrate variables like `appWindow`:
```ts
import { appWindow } from '@tauri-apps/api/window'
```
will become:
```ts
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
const appWindow = getCurrentWebviewWindow()
```
