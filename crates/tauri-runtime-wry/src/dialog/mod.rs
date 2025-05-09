#[cfg(windows)]
mod windows;

pub fn error<S: AsRef<str>>(err: S) {
  #[cfg(windows)]
  windows::error(err);

  #[cfg(not(windows))]
  {
    unimplemented!("Error dialog is not implemented for this platform");
  }
}
