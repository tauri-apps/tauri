---
"tauri": patch
"tauri-codegen": patch
"tauri-macros": patch
"tauri-utils": patch
---

Replace multiple dependencies who's C code compiled concurrently and caused
the other ones to bloat compile time significantly.

* `zstd` -> `brotli`
* `blake3` -> a vendored version of the blake3 reference
* `ring` -> `getrandom`

See https://github.com/tauri-apps/tauri/pull/3773 for more information about
these specific choices.
