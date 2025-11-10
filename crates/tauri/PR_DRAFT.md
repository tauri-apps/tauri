Title: Add kill-tree helper and runtime sidecar PID registry (Fixes #14360)

Summary

This PR adds a best-effort process-tree killer and a lightweight runtime registry to help ensure that descendant processes spawned by sidecars are terminated when a sidecar is killed.

What I changed

- crates/tauri/src/process.rs
  - Added `pub fn kill_process_tree(pid: u32) -> std::io::Result<()>` which invokes platform-specific shell/PowerShell snippets to terminate a process tree (best-effort, no new deps).

- crates/tauri/src/manager/mod.rs
  - Added `sidecar_pids: Arc<Mutex<HashSet<u32>>>` and methods: `register_sidecar`, `unregister_sidecar`, `drain_sidecar_pids`.

- crates/tauri/src/app.rs
  - Added `AppHandle::register_sidecar` and `AppHandle::unregister_sidecar` convenience methods.
  - Wired `cleanup_before_exit()` to drain the sidecar registry and call `kill_process_tree` for each registered PID.

- crates/tauri-cli/src/interface/rust/desktop.rs
  - Dev-run kill path now attempts a best-effort kill-tree invocation for dev child processes.

Testing done

- Ran `cargo test -p tauri` locally: unit tests and doc-tests passed.
- The changes are designed to be best-effort (no panics on failures). The runtime requires spawners to call `register_sidecar(pid)` after spawning a sidecar so it can be cleaned up at exit.

Notes & follow-ups

- This is a pragmatic, short-term fix using shell/PowerShell helpers. We can consider a Rust-native implementation later for better portability and finer control.
- We should update any sidecar spawners (plugins or examples that call `Command::spawn()`) to call `app.handle().register_sidecar(child.id() as u32)` after spawning and `unregister_sidecar` when stopping the sidecar.
- Add an integration test that spawns a parent process which itself spawns a child, registers the parent PID, and asserts both are gone after cleanup.

References

- Fixes #14360
