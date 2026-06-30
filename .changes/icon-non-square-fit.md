---
"tauri-cli": "minor:feat"
---

Add a `--fit` option to `tauri icon` to accept non-square source images. `--fit cover` center-crops the source to a square (clipping the longer side) and `--fit contain` pads the shorter side with transparency. Non-square sources without `--fit` keep erroring, now with a hint pointing to the flag.
