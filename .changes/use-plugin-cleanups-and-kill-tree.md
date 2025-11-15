---
'@tauri-apps/tauri': 'minor:enhance'
---

Introduce plugin-level cleanup hooks and centralize process-tree termination logic.

This change:
- Adds `kill_process_tree` helper to the runtime for cross-platform process-tree shutdown.
- Adds a new `cleanup_before_exit` lifecycle hook to plugins and wires it so plugin authors
  can handle sidecar shutdown without core runtime logic.
- Removes hardcoded sidecar-draining from the runtime and delegates shutdown behavior to plugins.

This allows plugins (such as the shell plugin) to manage their own sidecar processes cleanly
and improves extensibility of the Tauri runtime. Fixes #14360.
