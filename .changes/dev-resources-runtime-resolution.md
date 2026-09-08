---
"tauri": "minor:feat"
"tauri-build": "minor:feat"
"tauri-utils": "minor:feat"
"tauri-codegen": "minor:feat"
---

`tauri-build` no longer copies the configured resources to the cargo target directory; on desktop, unbundled apps (`tauri dev` / `cargo run`) now resolve resources at runtime from their source paths instead. This means editing a resource file no longer triggers a full application rebuild, and changes to plain relative resources are picked up live by the running app.

- When all configured resources are plain relative paths (e.g. `"assets/*"`), the resource directory resolves to the directory containing `tauri.conf.json` and files are read directly from the sources.
- When resources are remapped (map notation, `../` or absolute paths), the bundle layout is mirrored next to the executable on the first resource directory access of each run.

The `bundle > resources` configuration is now part of the config embedded by `generate_context!`, where it was previously stripped.

