---
"tauri": patch
"tauri-runtime-wry": patch
---

Removes `image` dependency. For now only `.ico` icons on Windows are supported, and we'll implement other types on demand to optimize bundle size.
