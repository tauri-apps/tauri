---
"tauri-macos-sign": patch:bug
"tauri-bundler": patch:bug
---

Sign macOS binaries with an explicit 4 KB code-signing page size (`codesign --pagesize 4096`). On Apple Silicon, `codesign` may emit a signature that uses a 16 KB page size, which macOS 26 (Tahoe)'s AMFI fails to load for larger binaries, killing the bundled app at launch with "Attempt to execute completely unsigned code" even though `codesign --verify` passes.
