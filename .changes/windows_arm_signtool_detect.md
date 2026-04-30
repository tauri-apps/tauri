---
"tauri-bundler": patch:enhance
---

Signtool path for windows arm systems was not being properly returned which caused failure in signing of windows binaries.

This patch addresses it.

Previously only the following were supported:

- PROCESSOR_ARCHITECTURE_INTEL
- PROCESSOR_ARCHITECTURE_AMD64

The following were added:

- PROCESSOR_ARCHITECTURE_ARM
- PROCESSOR_ARCHITECTURE_ARM64
