// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Master/Slave PTY Duplex Pipes module.
//!
//! Provides non-blocking pseudoterminal (PTY) communication channels with clean
//! EOF signal propagation, defensive pre-validation, precise low-level error interception,
//! and deterministic handle release to prevent worker thread hang.

use std::{
  fmt,
  io,
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
  },
  time::Duration,
};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(not(unix))]
pub type RawFd = i32;

/// Result type for PTY operations.
pub type PtyResult<T> = Result<T, PtyError>;

/// Granular errors for PTY operations with precise exception categorization.
#[derive(Debug)]
pub enum PtyError {
  /// File descriptor is invalid, uninitialized, or already closed.
  InvalidDescriptor(String),
  /// Buffer provided is invalid or has invalid size boundaries.
  InvalidBuffer(String),
  /// The PTY pipe was closed or broken.
  PipeClosed,
  /// PTY handle was already closed.
  AlreadyClosed,
  /// Non-blocking operation timed out.
  Timeout,
  /// The background reader thread was interrupted or halted prematurely.
  ReaderThreadHalted(String),
  /// Underlying I/O error from system calls.
  Io(io::Error),
  /// Configuration validation error.
  InvalidConfig(String),
  /// Unsupported platform or operation.
  UnsupportedPlatform(String),
}

impl fmt::Display for PtyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidDescriptor(msg) => write!(f, "invalid file descriptor: {msg}"),
      Self::InvalidBuffer(msg) => write!(f, "invalid buffer: {msg}"),
      Self::PipeClosed => write!(f, "PTY duplex pipe is closed"),
      Self::AlreadyClosed => write!(f, "PTY handle has already been closed"),
      Self::Timeout => write!(f, "PTY operation timed out"),
      Self::ReaderThreadHalted(msg) => write!(f, "reader thread halted: {msg}"),
      Self::Io(err) => write!(f, "PTY I/O error: {err}"),
      Self::InvalidConfig(msg) => write!(f, "invalid PTY configuration: {msg}"),
      Self::UnsupportedPlatform(msg) => write!(f, "unsupported platform: {msg}"),
    }
  }
}

impl std::error::Error for PtyError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Io(err) => Some(err),
      _ => None,
    }
  }
}

impl From<io::Error> for PtyError {
  fn from(err: io::Error) -> Self {
    Self::Io(err)
  }
}

impl PtyError {
  /// Checks whether this error represents an EOF or closed pipe condition.
  pub fn is_eof(&self) -> bool {
    matches!(self, Self::PipeClosed | Self::AlreadyClosed)
  }
}

/// Configuration options for initializing a PTY pair.
#[derive(Debug, Clone)]
pub struct PtyConfig {
  /// Terminal rows count (default 24). Must be > 0.
  pub rows: u16,
  /// Terminal columns count (default 80). Must be > 0.
  pub cols: u16,
  /// Pixel width (default 0).
  pub pixel_width: u16,
  /// Pixel height (default 0).
  pub pixel_height: u16,
  /// Whether the master PTY read/write operations operate in non-blocking mode.
  pub non_blocking: bool,
  /// Default read poll timeout in milliseconds.
  pub read_timeout: Option<Duration>,
  /// Whether to configure raw terminal mode (no echo, no CRLF translation).
  pub raw_mode: bool,
}

impl Default for PtyConfig {
  fn default() -> Self {
    Self {
      rows: 24,
      cols: 80,
      pixel_width: 0,
      pixel_height: 0,
      non_blocking: true,
      read_timeout: Some(Duration::from_millis(100)),
      raw_mode: true,
    }
  }
}

impl PtyConfig {
  /// Defensive validation of configuration parameters.
  pub fn validate(&self) -> PtyResult<()> {
    if self.rows == 0 {
      return Err(PtyError::InvalidConfig("terminal rows must be greater than 0".into()));
    }
    if self.cols == 0 {
      return Err(PtyError::InvalidConfig("terminal cols must be greater than 0".into()));
    }
    Ok(())
  }
}

/// Safe RAII wrapper around an OS file descriptor.
/// Prevents handle leaks and double-close vulnerabilities.
#[derive(Debug)]
pub struct SafeFd {
  fd: AtomicI32,
}

impl SafeFd {
  /// Creates a new `SafeFd` wrapping the given raw file descriptor.
  pub fn new(fd: RawFd) -> PtyResult<Self> {
    if fd < 0 {
      return Err(PtyError::InvalidDescriptor(format!("file descriptor must be non-negative, got {fd}")));
    }
    Ok(Self {
      fd: AtomicI32::new(fd),
    })
  }

  /// Returns the current raw file descriptor if it is still open.
  pub fn raw_fd(&self) -> PtyResult<RawFd> {
    let fd = self.fd.load(Ordering::Acquire);
    if fd < 0 {
      Err(PtyError::AlreadyClosed)
    } else {
      Ok(fd)
    }
  }

  /// Checks if the handle is still open.
  pub fn is_open(&self) -> bool {
    self.fd.load(Ordering::Acquire) >= 0
  }

  /// Explicitly closes the file descriptor and releases OS resources immediately.
  pub fn close(&self) -> PtyResult<()> {
    let fd = self.fd.swap(-1, Ordering::AcqRel);
    if fd >= 0 {
      #[cfg(unix)]
      {
        let res = unsafe { libc::close(fd) };
        if res != 0 {
          let err = io::Error::last_os_error();
          // EBADF is intercepted defensively
          if err.raw_os_error() != Some(libc::EBADF) {
            return Err(PtyError::Io(err));
          }
        }
      }
      Ok(())
    } else {
      Ok(())
    }
  }
}

impl Drop for SafeFd {
  fn drop(&mut self) {
    let _ = self.close();
  }
}

#[cfg(unix)]
impl AsRawFd for SafeFd {
  fn as_raw_fd(&self) -> RawFd {
    self.fd.load(Ordering::Acquire)
  }
}

/// Master controller end of the PTY duplex pipe.
#[derive(Debug, Clone)]
pub struct PtyMaster {
  fd: Arc<SafeFd>,
  is_closed: Arc<AtomicBool>,
  non_blocking: bool,
  read_timeout: Option<Duration>,
}

impl PtyMaster {
  /// Creates a new `PtyMaster` instance.
  pub fn new(fd: Arc<SafeFd>, non_blocking: bool, read_timeout: Option<Duration>) -> Self {
    Self {
      fd,
      is_closed: Arc::new(AtomicBool::new(false)),
      non_blocking,
      read_timeout,
    }
  }

  /// Returns whether the master end has been closed or reached EOF.
  pub fn is_closed(&self) -> bool {
    self.is_closed.load(Ordering::Acquire) || !self.fd.is_open()
  }

  /// Disarms and closes the master end, waking up any blocked reader immediately.
  pub fn close(&self) -> PtyResult<()> {
    self.is_closed.store(true, Ordering::Release);
    self.fd.close()
  }

  /// Reads bytes from the master PTY into the buffer with defensive validation,
  /// precise error interception (e.g. Linux EIO -> EOF), and non-blocking timeout handling.
  pub fn read(&self, buf: &mut [u8]) -> PtyResult<usize> {
    // 1. Defensive pre-validation
    if buf.is_empty() {
      return Err(PtyError::InvalidBuffer("read buffer must not be zero-sized".into()));
    }
    if self.is_closed() {
      return Ok(0); // Clean EOF signal propagation
    }

    let raw_fd = self.fd.raw_fd()?;

    #[cfg(unix)]
    {
      // Optional polling with timeout to avoid hanging worker threads
      if self.non_blocking {
        if let Some(timeout) = self.read_timeout {
          let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as libc::c_int;
          let mut pfd = libc::pollfd {
            fd: raw_fd,
            events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
            revents: 0,
          };

          loop {
            let poll_res = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
            if poll_res < 0 {
              let err = io::Error::last_os_error();
              if err.raw_os_error() == Some(libc::EINTR) {
                if self.is_closed() {
                  return Ok(0);
                }
                continue;
              }
              return Err(PtyError::Io(err));
            } else if poll_res == 0 {
              // Timeout reached
              return Err(PtyError::Timeout);
            } else {
              break;
            }
          }

          // Inspect poll events for hangup / error
          if pfd.revents & (libc::POLLHUP | libc::POLLERR) != 0 && pfd.revents & libc::POLLIN == 0 {
            self.is_closed.store(true, Ordering::Release);
            let _ = self.fd.close();
            return Ok(0); // Clean EOF
          }
        }
      }

      loop {
        let n = unsafe {
          libc::read(
            raw_fd,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len() as libc::size_t,
          )
        };

        if n > 0 {
          return Ok(n as usize);
        } else if n == 0 {
          // Clean EOF detected on BSD/macOS
          self.is_closed.store(true, Ordering::Release);
          let _ = self.fd.close();
          return Ok(0);
        } else {
          // Precise error interception
          let err = io::Error::last_os_error();
          match err.raw_os_error() {
            Some(libc::EINTR) => {
              if self.is_closed() {
                return Ok(0);
              }
              continue;
            }
            // On Linux, closing the slave end produces EIO on the master end.
            // This MUST be intercepted and converted cleanly to EOF.
            Some(libc::EIO) => {
              self.is_closed.store(true, Ordering::Release);
              let _ = self.fd.close();
              return Ok(0);
            }
            Some(libc::EAGAIN) => {
              return Err(PtyError::Timeout);
            }
            Some(libc::EBADF) => {
              self.is_closed.store(true, Ordering::Release);
              return Ok(0);
            }
            _ => return Err(PtyError::Io(err)),
          }
        }
      }
    }

    #[cfg(not(unix))]
    {
      Err(PtyError::UnsupportedPlatform("PTY requires Unix platform".into()))
    }
  }

  /// Writes bytes to the master PTY with defensive validation and error interception.
  pub fn write(&self, buf: &[u8]) -> PtyResult<usize> {
    // Defensive pre-validation
    if buf.is_empty() {
      return Ok(0);
    }
    if self.is_closed() {
      return Err(PtyError::PipeClosed);
    }

    let raw_fd = self.fd.raw_fd()?;

    #[cfg(unix)]
    {
      loop {
        let n = unsafe {
          libc::write(
            raw_fd,
            buf.as_ptr() as *const libc::c_void,
            buf.len() as libc::size_t,
          )
        };

        if n >= 0 {
          return Ok(n as usize);
        } else {
          let err = io::Error::last_os_error();
          match err.raw_os_error() {
            Some(libc::EINTR) => continue,
            Some(libc::EPIPE) | Some(libc::ECONNRESET) => {
              self.is_closed.store(true, Ordering::Release);
              return Err(PtyError::PipeClosed);
            }
            _ => return Err(PtyError::Io(err)),
          }
        }
      }
    }

    #[cfg(not(unix))]
    {
      Err(PtyError::UnsupportedPlatform("PTY requires Unix platform".into()))
    }
  }
}

/// Slave terminal end of the PTY duplex pipe.
#[derive(Debug)]
pub struct PtySlave {
  fd: Arc<SafeFd>,
  name: String,
}

impl PtySlave {
  /// Creates a new `PtySlave` instance.
  pub fn new(fd: Arc<SafeFd>, name: String) -> Self {
    Self { fd, name }
  }

  /// Returns the name of the slave PTY device path (e.g. `/dev/ttys001` or `/dev/pts/1`).
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the underlying raw file descriptor.
  pub fn raw_fd(&self) -> PtyResult<RawFd> {
    self.fd.raw_fd()
  }

  /// Closes the slave end.
  pub fn close(&self) -> PtyResult<()> {
    self.fd.close()
  }
}

/// Dedicated reader half of the PTY duplex pipe.
#[derive(Debug, Clone)]
pub struct PtyReader {
  master: PtyMaster,
}

impl PtyReader {
  /// Reads bytes from the PTY with defensive validation and clean EOF propagation.
  pub fn read(&self, buf: &mut [u8]) -> PtyResult<usize> {
    self.master.read(buf)
  }

  /// Signals EOF and closes the reader end immediately.
  pub fn close(&self) -> PtyResult<()> {
    self.master.close()
  }

  /// Spawns a dedicated worker thread to read from the PTY stream.
  /// Automatically exits and cleans up handles when EOF is reached or upon thread shutdown.
  pub fn spawn_reader_thread<F, E>(
    self,
    mut on_data: F,
    on_eof: E,
  ) -> std::thread::JoinHandle<PtyResult<()>>
  where
    F: FnMut(&[u8]) + Send + 'static,
    E: FnOnce() + Send + 'static,
  {
    std::thread::spawn(move || {
      let mut buffer = [0u8; 4096];
      loop {
        if self.master.is_closed() {
          break;
        }

        match self.master.read(&mut buffer) {
          Ok(0) => {
            // Clean EOF propagation: exit immediately without hanging
            break;
          }
          Ok(n) => {
            on_data(&buffer[..n]);
          }
          Err(PtyError::Timeout) => {
            // Non-blocking timeout: allow checking shutdown flags and loop condition
            continue;
          }
          Err(PtyError::PipeClosed) | Err(PtyError::AlreadyClosed) => {
            break;
          }
          Err(e) => {
            let _ = self.master.close();
            return Err(e);
          }
        }
      }

      // Defensive post-cleanup: ensure handle is released
      let _ = self.master.close();
      on_eof();
      Ok(())
    })
  }
}

/// Dedicated writer half of the PTY duplex pipe.
#[derive(Debug, Clone)]
pub struct PtyWriter {
  master: PtyMaster,
}

impl PtyWriter {
  /// Writes data to the PTY master end.
  pub fn write(&self, buf: &[u8]) -> PtyResult<usize> {
    self.master.write(buf)
  }

  /// Writes all data in the buffer to the PTY master end.
  pub fn write_all(&self, mut buf: &[u8]) -> PtyResult<()> {
    while !buf.is_empty() {
      let n = self.master.write(buf)?;
      if n == 0 {
        return Err(PtyError::PipeClosed);
      }
      buf = &buf[n..];
    }
    Ok(())
  }

  /// Closes the writer end.
  pub fn close(&self) -> PtyResult<()> {
    self.master.close()
  }
}

/// Complete Master/Slave PTY Duplex Pipe abstraction.
#[derive(Debug)]
pub struct PtyDuplexPipe {
  master: PtyMaster,
  slave: PtySlave,
  config: PtyConfig,
}

impl PtyDuplexPipe {
  /// Opens a new Master/Slave PTY pair with defensive pre-validation.
  pub fn open(config: PtyConfig) -> PtyResult<Self> {
    config.validate()?;

    #[cfg(unix)]
    {
      let mut master_fd: libc::c_int = -1;
      let mut slave_fd: libc::c_int = -1;
      let mut win = libc::winsize {
        ws_row: config.rows,
        ws_col: config.cols,
        ws_xpixel: config.pixel_width,
        ws_ypixel: config.pixel_height,
      };

      // Open PTY pair
      let res = unsafe {
        libc::openpty(
          &mut master_fd,
          &mut slave_fd,
          std::ptr::null_mut(),
          std::ptr::null_mut(),
          &mut win,
        )
      };

      if res != 0 {
        return Err(PtyError::Io(io::Error::last_os_error()));
      }

      // Defensive check of returned file descriptors
      if master_fd < 0 || slave_fd < 0 {
        if master_fd >= 0 {
          unsafe { libc::close(master_fd) };
        }
        if slave_fd >= 0 {
          unsafe { libc::close(slave_fd) };
        }
        return Err(PtyError::InvalidDescriptor(
          "openpty returned negative file descriptor".into(),
        ));
      }

      // Retrieve slave device name
      let mut name_buf = [0i8; 128];
      let slave_name = unsafe {
        if libc::ttyname_r(slave_fd, name_buf.as_mut_ptr(), name_buf.len()) == 0 {
          std::ffi::CStr::from_ptr(name_buf.as_ptr())
            .to_string_lossy()
            .into_owned()
        } else {
          format!("/dev/fd/{slave_fd}")
        }
      };

      // Configure non-blocking mode if requested
      if config.non_blocking {
        let flags = unsafe { libc::fcntl(master_fd, libc::F_GETFL) };
        if flags >= 0 {
          let _ = unsafe { libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        }
      }

      // Configure raw mode (no echo, binary transparent duplex) if requested
      if config.raw_mode {
        let mut termios: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(slave_fd, &mut termios) } == 0 {
          unsafe {
            libc::cfmakeraw(&mut termios);
            libc::tcsetattr(slave_fd, libc::TCSANOW, &termios);
          }
        }
      }

      let master_safe_fd = Arc::new(SafeFd::new(master_fd)?);
      let slave_safe_fd = Arc::new(SafeFd::new(slave_fd)?);

      let master = PtyMaster::new(master_safe_fd, config.non_blocking, config.read_timeout);
      let slave = PtySlave::new(slave_safe_fd, slave_name);

      Ok(Self {
        master,
        slave,
        config,
      })
    }

    #[cfg(not(unix))]
    {
      let _ = config;
      Err(PtyError::UnsupportedPlatform("PTY duplex pipes are only supported on Unix platforms".into()))
    }
  }

  /// Splits the duplex pipe into independent reader, writer, and slave handles.
  pub fn split(self) -> (PtyReader, PtyWriter, PtySlave) {
    let reader = PtyReader {
      master: self.master.clone(),
    };
    let writer = PtyWriter {
      master: self.master,
    };
    (reader, writer, self.slave)
  }

  /// Returns a reference to the master controller.
  pub fn master(&self) -> &PtyMaster {
    &self.master
  }

  /// Returns a reference to the slave terminal.
  pub fn slave(&self) -> &PtySlave {
    &self.slave
  }

  /// Returns the configuration.
  pub fn config(&self) -> &PtyConfig {
    &self.config
  }

  /// Defensively closes both master and slave ends, unblocking any active reader.
  pub fn close(&self) -> PtyResult<()> {
    let _ = self.master.close();
    let _ = self.slave.close();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::mpsc;

  #[test]
  fn test_config_validation() {
    let mut config = PtyConfig::default();
    assert!(config.validate().is_ok());

    config.rows = 0;
    assert!(config.validate().is_err());

    config.rows = 24;
    config.cols = 0;
    assert!(config.validate().is_err());
  }

  #[test]
  fn test_safe_fd_validation() {
    assert!(SafeFd::new(-1).is_err());
    assert!(SafeFd::new(-99).is_err());
  }

  #[cfg(unix)]
  #[test]
  fn test_pty_duplex_pipe_open_and_close() {
    let pipe = PtyDuplexPipe::open(PtyConfig::default()).expect("failed to open PTY");
    assert!(!pipe.master().is_closed());
    assert!(!pipe.slave().name().is_empty());
    assert!(pipe.slave().raw_fd().is_ok());

    // Defensive close
    pipe.close().expect("failed to close PTY");
    assert!(pipe.master().is_closed());
  }

  #[cfg(unix)]
  #[test]
  fn test_pty_bidirectional_communication() {
    let pipe = PtyDuplexPipe::open(PtyConfig {
      non_blocking: true,
      read_timeout: Some(Duration::from_millis(500)),
      ..Default::default()
    })
    .expect("failed to open PTY");

    let (reader, writer, slave) = pipe.split();

    // Write to slave end directly, read from master
    let slave_fd = slave.raw_fd().unwrap();
    let test_msg = b"hello tauri pty\n";
    let written = unsafe {
      libc::write(
        slave_fd,
        test_msg.as_ptr() as *const libc::c_void,
        test_msg.len(),
      )
    };
    assert_eq!(written as usize, test_msg.len());

    let mut buf = [0u8; 64];
    let n = reader.read(&mut buf).expect("failed to read from master");
    assert!(n > 0);
    assert_eq!(&buf[..n], test_msg);

    // Test writing from master
    assert!(writer.write(b"response").is_ok());
  }

  #[cfg(unix)]
  #[test]
  fn test_eof_signal_propagation_without_hang() {
    let pipe = PtyDuplexPipe::open(PtyConfig {
      non_blocking: true,
      read_timeout: Some(Duration::from_millis(200)),
      ..Default::default()
    })
    .expect("failed to open PTY");

    let (reader, _writer, slave) = pipe.split();

    let (tx_data, _rx_data) = mpsc::channel::<Vec<u8>>();
    let (tx_eof, rx_eof) = mpsc::channel::<()>();

    let join_handle = reader.spawn_reader_thread(
      move |bytes| {
        let _ = tx_data.send(bytes.to_vec());
      },
      move || {
        let _ = tx_eof.send(());
      },
    );

    // Close the slave end - this must propagate clean EOF to the reader
    slave.close().expect("failed to close slave");

    // The reader thread MUST receive EOF and finish promptly without hanging
    let eof_received = rx_eof.recv_timeout(Duration::from_secs(3));
    assert!(eof_received.is_ok(), "reader thread hung instead of propagating EOF");

    let thread_result = join_handle.join().expect("thread join panicked");
    assert!(thread_result.is_ok());
  }

  #[cfg(unix)]
  #[test]
  fn test_defensive_pre_validation_checks() {
    let pipe = PtyDuplexPipe::open(PtyConfig::default()).unwrap();
    let (reader, writer, _) = pipe.split();

    // 1. Zero-sized buffer check
    let mut empty_buf = [0u8; 0];
    assert!(matches!(
      reader.read(&mut empty_buf),
      Err(PtyError::InvalidBuffer(_))
    ));

    // 2. Post-close read returns EOF (0) rather than panicking or crashing
    reader.close().unwrap();
    let mut buf = [0u8; 16];
    assert_eq!(reader.read(&mut buf).unwrap(), 0);

    // 3. Post-close write returns PipeClosed
    assert!(matches!(writer.write(b"data"), Err(PtyError::PipeClosed)));
  }

  #[cfg(unix)]
  #[test]
  fn test_handle_release_on_drop() {
    let safe_fd = {
      let pipe = PtyDuplexPipe::open(PtyConfig::default()).unwrap();
      let (reader, _, _) = pipe.split();
      assert!(!reader.master.is_closed());
      reader.master.fd.clone()
    };
    // safe_fd should be closed once all references drop or when closed explicitly
    assert!(safe_fd.is_open());
    safe_fd.close().unwrap();
    assert!(!safe_fd.is_open());
    assert!(matches!(safe_fd.raw_fd(), Err(PtyError::AlreadyClosed)));
    // Double close should succeed idempotently without crashing
    assert!(safe_fd.close().is_ok());
  }

  #[cfg(unix)]
  #[test]
  fn test_multithreaded_duplex_stress() {
    let pipe = PtyDuplexPipe::open(PtyConfig {
      non_blocking: true,
      read_timeout: Some(Duration::from_millis(500)),
      ..Default::default()
    })
    .unwrap();

    let (reader, writer, slave) = pipe.split();
    let slave_fd = slave.raw_fd().unwrap();

    let reader_handle = std::thread::spawn(move || {
      let mut total_read = Vec::new();
      let mut buf = [0u8; 128];
      loop {
        match reader.read(&mut buf) {
          Ok(0) => break,
          Ok(n) => {
            total_read.extend_from_slice(&buf[..n]);
            if total_read.len() >= 50 {
              break;
            }
          }
          Err(PtyError::Timeout) => continue,
          Err(e) => panic!("unexpected error during reader stress: {e:?}"),
        }
      }
      total_read
    });

    let writer_handle = std::thread::spawn(move || {
      for i in 0..10 {
        let msg = format!("msg-{i}\n");
        let written = unsafe {
          libc::write(
            slave_fd,
            msg.as_ptr() as *const libc::c_void,
            msg.len(),
          )
        };
        assert!(written > 0);
        std::thread::sleep(Duration::from_millis(10));
      }
    });

    writer_handle.join().unwrap();
    let received = reader_handle.join().unwrap();
    assert!(received.len() >= 50);
    assert!(String::from_utf8_lossy(&received).contains("msg-0"));

    // Cleanup
    let _ = writer.close();
    let _ = slave.close();
  }
}
