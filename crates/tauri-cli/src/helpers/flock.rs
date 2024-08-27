// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/rust-lang/cargo/blob/b0c9586f4cbf426914df47c65de38ea323772c74/src/cargo/util/flock.rs
#![allow(dead_code)]

use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::Result;
use anyhow::Context as _;
use sys::*;

#[derive(Debug)]
pub struct FileLock {
  f: Option<File>,
  path: PathBuf,
  state: State,
}

#[derive(PartialEq, Debug)]
enum State {
  Unlocked,
  Shared,
  Exclusive,
}

impl FileLock {
  /// Returns the underlying file handle of this lock.
  pub fn file(&self) -> &File {
    self.f.as_ref().unwrap()
  }

  /// Returns the underlying path that this lock points to.
  ///
  /// Note that special care must be taken to ensure that the path is not
  /// referenced outside the lifetime of this lock.
  pub fn path(&self) -> &Path {
    assert_ne!(self.state, State::Unlocked);
    &self.path
  }

  /// Returns the parent path containing this file
  pub fn parent(&self) -> &Path {
    assert_ne!(self.state, State::Unlocked);
    self.path.parent().unwrap()
  }
}

impl Read for FileLock {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.file().read(buf)
  }
}

impl Seek for FileLock {
  fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
    self.file().seek(to)
  }
}

impl Write for FileLock {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.file().write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file().flush()
  }
}

impl Drop for FileLock {
  fn drop(&mut self) {
    if self.state != State::Unlocked {
      if let Some(f) = self.f.take() {
        let _ = unlock(&f);
      }
    }
  }
}

/// Opens exclusive access to a file, returning the locked version of a
/// file.
///
/// This function will create a file at `path` if it doesn't already exist
/// (including intermediate directories), and then it will acquire an
/// exclusive lock on `path`. If the process must block waiting for the
/// lock, the `msg` is logged.
///
/// The returned file can be accessed to look at the path and also has
/// read/write access to the underlying file.
pub fn open_rw<P>(path: P, msg: &str) -> Result<FileLock>
where
  P: AsRef<Path>,
{
  open(
    path.as_ref(),
    OpenOptions::new().read(true).write(true).create(true),
    State::Exclusive,
    msg,
  )
}

/// Opens shared access to a file, returning the locked version of a file.
///
/// This function will fail if `path` doesn't already exist, but if it does
/// then it will acquire a shared lock on `path`. If the process must block
/// waiting for the lock, the `msg` is logged.
///
/// The returned file can be accessed to look at the path and also has read
/// access to the underlying file. Any writes to the file will return an
/// error.
pub fn open_ro<P>(path: P, msg: &str) -> Result<FileLock>
where
  P: AsRef<Path>,
{
  open(
    path.as_ref(),
    OpenOptions::new().read(true),
    State::Shared,
    msg,
  )
}

fn open(path: &Path, opts: &OpenOptions, state: State, msg: &str) -> Result<FileLock> {
  // If we want an exclusive lock then if we fail because of NotFound it's
  // likely because an intermediate directory didn't exist, so try to
  // create the directory and then continue.
  let f = opts
    .open(path)
    .or_else(|e| {
      if e.kind() == io::ErrorKind::NotFound && state == State::Exclusive {
        create_dir_all(path.parent().unwrap())?;
        Ok(opts.open(path)?)
      } else {
        Err(anyhow::Error::from(e))
      }
    })
    .with_context(|| format!("failed to open: {}", path.display()))?;
  match state {
    State::Exclusive => {
      acquire(msg, path, &|| try_lock_exclusive(&f), &|| {
        lock_exclusive(&f)
      })?;
    }
    State::Shared => {
      acquire(msg, path, &|| try_lock_shared(&f), &|| lock_shared(&f))?;
    }
    State::Unlocked => {}
  }
  Ok(FileLock {
    f: Some(f),
    path: path.to_path_buf(),
    state,
  })
}

/// Acquires a lock on a file in a "nice" manner.
///
/// Almost all long-running blocking actions in Cargo have a status message
/// associated with them as we're not sure how long they'll take. Whenever a
/// conflicted file lock happens, this is the case (we're not sure when the lock
/// will be released).
///
/// This function will acquire the lock on a `path`, printing out a nice message
/// to the console if we have to wait for it. It will first attempt to use `try`
/// to acquire a lock on the crate, and in the case of contention it will emit a
/// status message based on `msg` to `config`'s shell, and then use `block` to
/// block waiting to acquire a lock.
///
/// Returns an error if the lock could not be acquired or if any error other
/// than a contention error happens.
fn acquire(
  msg: &str,
  path: &Path,
  lock_try: &dyn Fn() -> io::Result<()>,
  lock_block: &dyn Fn() -> io::Result<()>,
) -> Result<()> {
  // File locking on Unix is currently implemented via `flock`, which is known
  // to be broken on NFS. We could in theory just ignore errors that happen on
  // NFS, but apparently the failure mode [1] for `flock` on NFS is **blocking
  // forever**, even if the "non-blocking" flag is passed!
  //
  // As a result, we just skip all file locks entirely on NFS mounts. That
  // should avoid calling any `flock` functions at all, and it wouldn't work
  // there anyway.
  //
  // [1]: https://github.com/rust-lang/cargo/issues/2615
  if is_on_nfs_mount(path) {
    return Ok(());
  }

  match lock_try() {
    Ok(()) => return Ok(()),

    // In addition to ignoring NFS which is commonly not working we also
    // just ignore locking on filesystems that look like they don't
    // implement file locking.
    Err(e) if error_unsupported(&e) => return Ok(()),

    Err(e) => {
      if !error_contended(&e) {
        let e = anyhow::Error::from(e);
        let cx = format!("failed to lock file: {}", path.display());
        return Err(e.context(cx));
      }
    }
  }
  let msg = format!("waiting for file lock on {msg}");
  log::info!(action = "Blocking"; "{}", &msg);

  lock_block().with_context(|| format!("failed to lock file: {}", path.display()))?;
  return Ok(());

  #[cfg(all(target_os = "linux", not(target_env = "musl")))]
  fn is_on_nfs_mount(path: &Path) -> bool {
    use std::ffi::CString;
    use std::mem;
    use std::os::unix::prelude::*;

    let path = match CString::new(path.as_os_str().as_bytes()) {
      Ok(path) => path,
      Err(_) => return false,
    };

    unsafe {
      let mut buf: libc::statfs = mem::zeroed();
      let r = libc::statfs(path.as_ptr(), &mut buf);

      r == 0 && buf.f_type as u32 == libc::NFS_SUPER_MAGIC as u32
    }
  }

  #[cfg(any(not(target_os = "linux"), target_env = "musl"))]
  fn is_on_nfs_mount(_path: &Path) -> bool {
    false
  }
}

#[cfg(unix)]
mod sys {
  use std::fs::File;
  use std::io::{Error, Result};
  use std::os::unix::io::AsRawFd;

  pub(super) fn lock_shared(file: &File) -> Result<()> {
    flock(file, libc::LOCK_SH)
  }

  pub(super) fn lock_exclusive(file: &File) -> Result<()> {
    flock(file, libc::LOCK_EX)
  }

  pub(super) fn try_lock_shared(file: &File) -> Result<()> {
    flock(file, libc::LOCK_SH | libc::LOCK_NB)
  }

  pub(super) fn try_lock_exclusive(file: &File) -> Result<()> {
    flock(file, libc::LOCK_EX | libc::LOCK_NB)
  }

  pub(super) fn unlock(file: &File) -> Result<()> {
    flock(file, libc::LOCK_UN)
  }

  pub(super) fn error_contended(err: &Error) -> bool {
    err.raw_os_error().map_or(false, |x| x == libc::EWOULDBLOCK)
  }

  pub(super) fn error_unsupported(err: &Error) -> bool {
    match err.raw_os_error() {
      // Unfortunately, depending on the target, these may or may not be the same.
      // For targets in which they are the same, the duplicate pattern causes a warning.
      #[allow(unreachable_patterns)]
      Some(libc::ENOTSUP | libc::EOPNOTSUPP) => true,
      Some(libc::ENOSYS) => true,
      _ => false,
    }
  }

  #[cfg(not(target_os = "solaris"))]
  fn flock(file: &File, flag: libc::c_int) -> Result<()> {
    let ret = unsafe { libc::flock(file.as_raw_fd(), flag) };
    if ret < 0 {
      Err(Error::last_os_error())
    } else {
      Ok(())
    }
  }

  #[cfg(target_os = "solaris")]
  fn flock(file: &File, flag: libc::c_int) -> Result<()> {
    // Solaris lacks flock(), so simply succeed with a no-op
    Ok(())
  }
}

#[cfg(windows)]
mod sys {
  use std::fs::File;
  use std::io::{Error, Result};
  use std::mem;
  use std::os::windows::io::AsRawHandle;

  use windows_sys::Win32::Foundation::{ERROR_INVALID_FUNCTION, ERROR_LOCK_VIOLATION, HANDLE};
  use windows_sys::Win32::Storage::FileSystem::{
    LockFileEx, UnlockFile, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LOCK_FILE_FLAGS,
  };

  pub(super) fn lock_shared(file: &File) -> Result<()> {
    lock_file(file, 0)
  }

  pub(super) fn lock_exclusive(file: &File) -> Result<()> {
    lock_file(file, LOCKFILE_EXCLUSIVE_LOCK)
  }

  pub(super) fn try_lock_shared(file: &File) -> Result<()> {
    lock_file(file, LOCKFILE_FAIL_IMMEDIATELY)
  }

  pub(super) fn try_lock_exclusive(file: &File) -> Result<()> {
    lock_file(file, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY)
  }

  pub(super) fn error_contended(err: &Error) -> bool {
    err
      .raw_os_error()
      .map_or(false, |x| x == ERROR_LOCK_VIOLATION as i32)
  }

  pub(super) fn error_unsupported(err: &Error) -> bool {
    err
      .raw_os_error()
      .map_or(false, |x| x == ERROR_INVALID_FUNCTION as i32)
  }

  pub(super) fn unlock(file: &File) -> Result<()> {
    let ret = unsafe { UnlockFile(file.as_raw_handle() as HANDLE, 0, 0, !0, !0) };
    if ret == 0 {
      Err(Error::last_os_error())
    } else {
      Ok(())
    }
  }

  fn lock_file(file: &File, flags: LOCK_FILE_FLAGS) -> Result<()> {
    let ret = unsafe {
      let mut overlapped = mem::zeroed();
      LockFileEx(
        file.as_raw_handle() as HANDLE,
        flags,
        0,
        !0,
        !0,
        &mut overlapped,
      )
    };

    if ret == 0 {
      Err(Error::last_os_error())
    } else {
      Ok(())
    }
  }
}
