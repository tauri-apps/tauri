---
"@tauri-apps/cli": patch:enhance
"tauri-cli": patch:enhance
---

Support custom project directory structure where the Tauri app folder is not a subfolder of the frontend project.
The frontend and Tauri app project paths can be set with the `TAURI_FRONTEND_PATH` and the `TAURI_APP_PATH` environment variables respectively.
