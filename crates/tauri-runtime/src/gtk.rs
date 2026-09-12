// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The GTK version a runtime binds to on Linux and BSD.
//!
//! `tauri` binds one GTK version at compile time (its `gtk3`/`gtk4` features) while the runtime is
//! picked at run time, and the pointers crossing [`WindowDispatch::gtk_window`] and friends carry
//! no version information. A runtime declares the version its objects belong to with
//! [`declare_version`] so `tauri` can refuse to wrap them with mismatched bindings instead of
//! reinterpreting a GTK 3 object as a GTK 4 one.
//!
//! Note that GTK 3 and GTK 4 cannot be initialized in the same process - GTK 4 aborts when it
//! detects GTK 2/3 symbols - so a single binary can only ever run one of them.
//!
//! [`WindowDispatch::gtk_window`]: crate::WindowDispatch::gtk_window

use std::sync::atomic::{AtomicU8, Ordering};

/// GTK major version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
  /// GTK 3.
  V3 = 3,
  /// GTK 4.
  V4 = 4,
}

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::V3 => write!(f, "GTK 3"),
      Self::V4 => write!(f, "GTK 4"),
    }
  }
}

static ACTIVE_VERSION: AtomicU8 = AtomicU8::new(0);

/// Declares the GTK version this runtime's GTK object pointers belong to.
///
/// Runtimes must call this before creating any window. The last call wins; since GTK 3 and GTK 4
/// cannot share a process, a binary that declares both is already unable to run.
pub fn declare_version(version: Version) {
  ACTIVE_VERSION.store(version as u8, Ordering::Relaxed);
}

/// The GTK version declared by the active runtime, or [`None`] if no runtime declared one.
pub fn active_version() -> Option<Version> {
  match ACTIVE_VERSION.load(Ordering::Relaxed) {
    3 => Some(Version::V3),
    4 => Some(Version::V4),
    _ => None,
  }
}
