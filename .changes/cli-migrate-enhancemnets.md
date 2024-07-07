---
"tauri-cli": "patch:enhance"
"@tauri-apps/cli": "patch:enhance"
---

Enhance `tauri migrate` to also migrate variables like `appWindow`:
```ts
import { appWindow } from '@tauri-apps/api/window'
```
will become:
```
import { getCurrent } from '@tauri-apps/api/window'
const appWindow = getCurrent()
```
