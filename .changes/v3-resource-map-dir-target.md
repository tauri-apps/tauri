---
"tauri-utils": major:breaking
"tauri-bundler": major:breaking
---

In the `bundle > resources` map, a target ending with a path separator (`/`, and also `\` on Windows) or a `.` segment now marks a directory the file is copied into, keeping its name: `"README.md": "docs/"` yields `$RESOURCE/docs/README.md` (it used to produce a file named `docs`). The empty target keeps working as before.
