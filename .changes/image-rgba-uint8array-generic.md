---
"@tauri-apps/api": patch:bug
---

`Image.rgba` now returns a more specific type `Promise<Uint8Array<ArrayBuffer>>` instead of the default `Promise<Uint8Array<ArrayBufferLike>`
