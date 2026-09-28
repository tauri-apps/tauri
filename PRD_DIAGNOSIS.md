# PRD: #2048: Refactor PTY master slave duplex pipes on Unix platforms

## 1. Background & Problem Statement
When working with pseudo-terminals (PTYs) on Unix platforms (Linux, macOS, BSD), the master side reads output produced by slave processes. However, behavior diverges significantly when the slave closes its file descriptor or terminates:
- On Linux, reading from a master PTY after slave closure produces an `EIO` error (errno 5) rather than the standard 0-byte EOF returned by BSD/macOS systems.
- Unmanaged blocking reads block reader threads permanently if the child process exits or closes without producing data, as there is no polling timeout or atomic cancellation mechanism.
- Direct raw file descriptor usage without RAII wrappers risks file descriptor leaks and undefined behavior from double-close.

## 2. Goals & Objectives
- Provide a safe, cross-platform Unix pseudo-terminal duplex pipe abstraction (`PtyDuplexPipe`).
- Normalize EOF behavior across Linux and macOS/BSD so that slave closure cleanly translates into 0 bytes (EOF) without bubbling up raw `EIO` errors.
- Prevent reader thread deadlocks and hangs by using non-blocking I/O with `libc::poll` and atomic shutdown flags.
- Provide an ergonomic background reader thread API (`spawn_reader_thread`) with `on_data` and `on_eof` callbacks.

## 3. Architecture & Key Components
- `SafeFd`: Thread-safe RAII file descriptor wrapper using `AtomicI32` ensuring idempotent closure.
- `PtyConfig`: Configuration structure for initial terminal window size (rows, cols) and raw/echo modes.
- `PtyError`: Strongly typed error enum covering `InvalidDescriptor`, `PipeClosed`, `Timeout`, and standard `Io`.
- `PtyMaster` & `PtySlave`: Master and slave handle abstractions wrapping `SafeFd`.
- `PtyReader` & `PtyWriter`: Split reader and writer handles for full-duplex communication.
- `spawn_reader_thread`: Helper spawning a background thread polling master FD and invoking user callbacks.

## 4. Acceptance Criteria & Test Plan
- Verification via `cargo test`.
- Unit tests validating PTY creation, window resize, write/read roundtrip, clean EOF handling on slave close (handling Linux `EIO` as EOF), and graceful reader thread termination.