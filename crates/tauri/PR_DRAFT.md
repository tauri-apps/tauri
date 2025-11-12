Title: Use plugin cleanup hooks for sidecar shutdown; add kill-tree helper (Fixes #14360)

Summary

This PR centralizes process-kill helpers in the runtime and moves responsibility for sidecar shutdown to plugins by adding a plugin-level cleanup hook. The core runtime no longer contains hardcoded sidecar draining logic; instead it calls `Plugin::cleanup_before_exit` so plugins (for example, the shell plugin) can take care of stopping sidecars and terminating process trees.

What I changed

- crates/tauri/src/process.rs
  - Added `pub fn kill_process_tree(pid: u32) -> std::io::Result<()>` which invokes platform-specific shell/PowerShell snippets to terminate a process tree (best-effort, no new deps).

- crates/tauri/src/plugin.rs
  - Added a plugin lifecycle hook `fn cleanup_before_exit(&mut self, app: &AppHandle<R>) {}` with a default no-op.
  - Added `PluginStore::cleanup_before_exit(&mut self, app: &AppHandle<R>)` which invokes the hook for every registered plugin.

- crates/tauri/src/app.rs
  - Replaced the hardcoded sidecar PID draining and kill logic in the app shutdown path with a call to `plugins.lock().unwrap().cleanup_before_exit(self.app_handle())` so plugins perform shutdown work.
  - Removed the previous `AppHandle::register_sidecar` / `unregister_sidecar` convenience methods from the public API (spawners should migrate to plugin-managed registries).

- Note on migration / shell plugin responsibilities
  - The shell plugin (which lives in the plugins workspace) should implement `cleanup_before_exit` and perform any sidecar shutdown it needs. A minimal implementation is to drain the manager's sidecar registry and call `kill_process_tree` for each PID, e.g.:

  ```text
  fn cleanup_before_exit(&mut self, app: &AppHandle<R>) {
    let pids = app.manager.drain_sidecar_pids();
    for pid in pids {
      let _ = tauri::process::kill_process_tree(pid);
    }
  }
  ```

  This keeps process-management details inside the shell plugin (owner of sidecar lifecycle) and makes the runtime more extensible.

Testing done

- Ran `cargo check -p tauri` locally to verify compilation after wiring the plugin hook (no compile errors; one doc warning for `kill_process_tree`).

Notes & follow-ups

- Implement `cleanup_before_exit` in the shell plugin (plugins-workspace repo). The shell plugin should be updated to drain any sidecar PID registries it manages and use `kill_process_tree` to ensure descendant processes are terminated.
- Update examples and any existing sidecar spawners to use the shell plugin or to call plugin-provided APIs instead of the removed `AppHandle::register_sidecar`/`unregister_sidecar`.
- Consider adding an integration test that validates sidecar shutdown via the shell plugin's cleanup hook.

References

- Fixes #14360
