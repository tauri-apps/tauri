---
"api": patch
---

* **Breaking Change** Renamed `tempdir` to `tmpdir` (it was a typo).
* **Breaking Change** Renamed all path functions in `path` module to be lowercase for example: `appDir` is now `appdir` (better allignment with NodeJS naming convention).
* You can acess `tmpdir` from `path` module now too.
* You can acess `homedir` from `os` module now too.
