---
"cli.rs": patch
---

On Windows, Fix `beforeDevCommand` and `beforeBuildCommand` not executing the expected command if it contains quotes. This is done by executing them with `CMD /S /C {command}` instead of `CMD /C {command}` on Windows.