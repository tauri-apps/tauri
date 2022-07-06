---
"cli.rs": patch
"cli.js": patch
---

Skip the static link of the `vcruntime140.dll` if the `STATIC_VCRUNTIME` environment variable is set to `false`.
