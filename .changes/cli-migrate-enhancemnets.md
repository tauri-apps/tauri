---
"tauri-cli": "patch:enhance"
"@tauri-apps/cli": "patch:enhance"
---

Enhance `tauri migrate` to also partially migrate variables like `appWindow`, so `import { appWindow } from '@tauri-apps/api/window'` becomes `import { getCurrent as appWindow } from '@tauri-apps/api/window' `
