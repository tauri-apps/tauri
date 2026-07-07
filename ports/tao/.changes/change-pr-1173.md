---
"tao": patch
---

`fn set_min_inner_size`, `fn set_max_inner_size`, `fn set_inner_size_constraints`,
`fn set_fullscreen` and `fn set_theme` on `Window` were not properly thread-safe
on Linux.
