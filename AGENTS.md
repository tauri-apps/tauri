# Task Workspace Context Protocol

## Workspace Sandbox Root
`/Users/fangqq/.bounty_agent_platform/workspaces/algora-tauri-2048`

## Task Info
- Repository: tauri-apps/tauri
- Issue: #2048
- Title: Refactor PTY master slave duplex pipes on Unix platforms

## Git Branch Protocol
- Feature Branch: `fix/issue-2048`
- Target Base Branch: `dev`

## Issue Description


## Step 2 PRD Specifications
- Root Cause: On Unix platforms (Linux, macOS, BSD), reading from a pseudoterminal (PTY) master descriptor when the slave process closes its end or terminates exhibits platform-specific behavior: Linux raises EIO (errno 5) rather than returning 0 bytes (EOF), whereas BSD/macOS returns 0 bytes. Furthermore, without non-blocking I/O (O_NONBLOCK) and poll/select polling for POLLHUP/POLLERR events, synchronous read operations hang worker threads indefinitely. Without thread-safe atomic state flags and deterministic RAII handle cleanup, closing the pipe fails to unblock the reader loop, leading to thread hangs and descriptor leaks.
- Proposed Fix: Implement a robust Unix Master/Slave PTY Duplex Pipe abstraction (`PtyDuplexPipe`, `PtyMaster`, `PtySlave`, `PtyReader`, `PtyWriter`) featuring: (1) Safe RAII file descriptor management (`SafeFd`) with atomic descriptor storage preventing handle leaks and double-close issues; (2) Non-blocking I/O configuration via `fcntl(O_NONBLOCK)` combined with `libc::poll` event filtering for `POLLIN`, `POLLHUP`, and `POLLERR`; (3) Precise error interception converting Linux `EIO`, `EBADF`, `POLLHUP`, and 0-byte reads into a clean EOF return (0 bytes) and atomic `is_closed` transition; (4) A dedicated reader thread worker (`spawn_reader_thread`) that exits immediately upon EOF or handle closure, invoking lifecycle callbacks (`on_data`, `on_eof`) without hanging.
- Test Command/Strategy: cargo test

## Demand-to-Code Mapping
- Requirement: Non-blocking Unix PTY master/slave duplex pipe initialization
  Affected: ["crates/tauri/src/pty.rs"]
  Action: Implement `PtyDuplexPipe::open` using `libc::openpty`, configuring `O_NONBLOCK` on master FD and raw mode on slave FD, encapsulated in thread-safe `SafeFd` handles.
- Requirement: Clean EOF signal propagation across Unix variants (Linux EIO and BSD/macOS EOF)
  Affected: ["crates/tauri/src/pty.rs"]
  Action: Implement error normalization in `PtyMaster::read` to catch `libc::EIO`, `libc::EBADF`, and `libc::POLLHUP`, cleanly transitioning state to closed and returning 0 (EOF).
- Requirement: Prevent reader worker thread hanging upon termination
  Affected: ["crates/tauri/src/pty.rs"]
  Action: Implement `PtyReader::spawn_reader_thread` with non-blocking poll timeouts, atomic cancellation checks, and deterministic post-loop FD closure and EOF notification.
- Requirement: Public API export and module integration
  Affected: ["crates/tauri/src/lib.rs", "crates/tauri/src/process.rs"]
  Action: Declare `pub mod pty` in `crates/tauri/src/lib.rs` and re-export `pty` in `crates/tauri/src/process.rs`.

## Actionable Task Checklist
1. Task 1: Define `PtyConfig` and `PtyError` enum with granular error variants (`InvalidDescriptor`, `PipeClosed`, `Timeout`, `Io`) and `is_eof()` helper in `crates/tauri/src/pty.rs`
2. Task 2: Implement RAII descriptor wrapper `SafeFd` with atomic operations and idempotent `close()` avoiding double-close vulnerabilities
3. Task 3: Implement `PtyDuplexPipe::open()` with `libc::openpty()`, window size configuration, `O_NONBLOCK` flags, and terminal raw mode setup
4. Task 4: Implement `PtyMaster::read()` and `PtyMaster::write()` with `libc::poll` readiness polling, non-blocking timeouts, and precise Linux `EIO` to EOF conversion
5. Task 5: Implement `PtyReader::spawn_reader_thread()` supporting `on_data` and `on_eof` callbacks with deterministic thread termination upon EOF or channel close
6. Task 6: Re-export `pty` module in `crates/tauri/src/lib.rs` and `crates/tauri/src/process.rs`
7. Task 7: Add unit and integration tests in `crates/tauri/src/pty.rs` covering configuration, bidirectional transmission, clean EOF propagation, and concurrent stress testing

## Strict Guidelines
1. All modifications MUST be made inside `/Users/fangqq/.bounty_agent_platform/workspaces/algora-tauri-2048`.
2. Implement complete code with ZERO stubs or placeholders.
3. Run `cargo test` to verify all tests pass.
