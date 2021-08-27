---
"api": patch
---

* **Breaking Change** Renamed `tempdir` to `tmpdir` (it was a typo).
* **Breaking Change** Removed all predefined-paths functions in `path` module to be lowercase for example: `appDir` is now `appdir` (better allignment with NodeJS naming convention and os module).
* You can acess `tmpdir` from `path` module now too.
* You can acess `homedir` from `os` module now too.
