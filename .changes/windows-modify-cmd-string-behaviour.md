---
"cli.rs": patch
---

Change `beforeDevCommand` and `beforeBuildCommand` to run with `CMD /S /C {command}` instead of `CMD /C {command}` on Windows.