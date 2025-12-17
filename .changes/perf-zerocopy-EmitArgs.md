---
"tauri": patch:perf
---

Change EmitArgs to hold RawValue rather than String to makes payload zerocopy. No user facing changes.
