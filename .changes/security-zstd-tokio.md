---
"tauri": "patch:bug"
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
"tauri-driver": "patch:bug"
---

Update the CLI's locked Zstandard dependencies to address RUSTSEC-2026-0391 and RUSTSEC-2026-0395 through RUSTSEC-2026-0406, plus RUSTSEC-2026-0410. Update the locked Tokio dependency to 1.53.3 to address RUSTSEC-2026-0386, an I/O safety issue in asynchronous file descriptor constructors.

Update JavaScript development dependencies to patched versions of `body-parser`, `fast-uri`, `serialize-javascript`, and both supported major versions of `undici`.
