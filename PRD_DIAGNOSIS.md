# PRD: #2048: Refactor PTY master slave duplex pipes on Unix platforms

## 1. Problem Statement & Root Cause Analysis
In Unix-like systems (Linux, macOS, BSD), pseudoterminal (PTY) communication relies on a master/slave bidirectional duplex pair. When the slave process exits or closes its file descriptor:
- On Linux, reading from the master PTY returns `-1` with `errno == EIO` (Input/Output Error), rather than returning `0` (clean EOF).
- On BSD and macOS, reading returns `0` bytes.
- When PTY file descriptors operate in blocking mode without polling timeouts, worker reader threads block indefinitely waiting for EOF or new input.
- Without explicit atomic state tracking and synchronized handle release, shutting down the PTY master or slave leaks descriptors and hangs background reader threads.

## 2. Technical Architecture & Component Design
### 2.1 RAII File Descriptor Management (`SafeFd`)
- Wraps raw Unix file descriptors in an atomic integer (`AtomicI32`).
- Ensures thread-safe access, prevents duplicate closes (`EBADF` handling), and implements `Drop` for deterministic resource reclamation.

### 2.2 PTY Master/Slave Duplex Pipe (`PtyDuplexPipe`)
- Initializes master and slave via `libc::openpty` with configurable rows and columns.
- Configures non-blocking I/O using `fcntl(master_fd, F_SETFL, O_NONBLOCK)`.
- Configures binary transparency with `libc::cfmakeraw` on the slave terminal descriptor.
- Provides a `.split()` method to produce independent `PtyReader`, `PtyWriter`, and `PtySlave` handles.

### 2.3 Non-Blocking Polling & Clean EOF Interception (`PtyMaster` & `PtyReader`)
- Uses `libc::poll` with timeout to prevent thread starvation.
- Detects `POLLHUP` and `POLLERR` disconnection flags and converts them immediately to clean EOF.
- Traps `libc::EIO`, `libc::EBADF`, and 0-byte reads, cleanly updating `is_closed` atomic state and returning `Ok(0)`.

### 2.4 Worker Reader Thread (`spawn_reader_thread`)
- Runs a dedicated background reader loop processing incoming data via `on_data`.
- Upon receiving EOF or pipe closure, immediately invokes `on_eof`, releases underlying descriptors, and terminates cleanly.

## 3. Verification & Test Plan
- **Config Validation**: Test invalid boundaries (0 rows, 0 cols) and invalid file descriptors.
- **Bidirectional I/O**: Verify data written to slave is read by master, and data written to master is received.
- **Clean EOF Propagation**: Simulate slave closure and assert that `spawn_reader_thread` receives EOF and terminates within 3 seconds.
- **Error Interception**: Verify that closing handles returns `Ok(0)` for subsequent reads rather than raising unhandled OS errors.
- **Multithreaded Stress**: Concurrently write from slave while reading from master across multiple threads to ensure thread-safety and absence of deadlocks.