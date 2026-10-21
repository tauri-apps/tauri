---
"tauri-bundler": patch:bug
---

Ship the Persian NSIS translation under `Farsi`, the name NSIS knows the language by. The toolset has no `Persian.nlf`, so `bundle > windows > nsis > languages` containing `Persian` loaded no language table and the translation referenced a `${LANG_PERSIAN}` nothing defined, failing the build. `Persian` is still accepted as a deprecated alias for `Farsi`.
