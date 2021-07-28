---
"api": patch
---

- Add new nodejs-inspired functions which are `join`, `resolve`, `normalize`, `dirname`, `basename` and `extname`.
- Add `sep` and `delimiter` constants.
- Deprecated `resolvePath` and will eventually be removed, use `resolve` and `join` instead.
