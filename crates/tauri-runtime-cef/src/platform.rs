//! Platform-specific window helpers for the CEF runtime.
//!
//! CEF's Views framework does not expose APIs for many window-manager
//! integrations (taskbar, cursor, progress, attention, ...), so for those we
//! reach down to the native window handle returned by
//! [`cef::Window::window_handle`] and talk to the platform directly, mirroring
//! what `tao` does for the wry backend.
//!
//! Each platform implements the same set of functions in its own file; a
//! function is a no-op on platforms where the feature does not exist (or where
//! CEF/the OS provides no way to implement it), documented inline there.

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
#[path = "platform_linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "platform_windows.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "platform_macos.rs"]
mod imp;

pub(crate) use imp::*;
