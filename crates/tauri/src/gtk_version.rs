// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Guards the GTK bindings selected at compile time against the runtime picked at run time.
//!
//! The `gtk3` and `gtk4` features are additive, so a build graph containing a GTK3 runtime
//! (`tauri-runtime-wry`) and a GTK4 one (`tauri-runtime-cef`) enables both and this crate binds
//! GTK 4. The GTK pointers coming back from the runtime carry no version information, so wrapping
//! a GTK 3 object with the GTK 4 bindings would reinterpret it. Compare the runtime's declared
//! version with ours instead and fail loudly.

use tauri_runtime::gtk::Version;

/// The GTK version the enabled features bound this crate to.
const COMPILED: Version = if cfg!(feature = "gtk4") {
  Version::V4
} else {
  Version::V3
};

/// Fails when the active runtime's GTK objects are not the version this crate binds.
pub(crate) fn check() -> crate::Result<()> {
  match tauri_runtime::gtk::active_version() {
    // no runtime declared a version: nothing to compare against, keep the previous behavior
    None => Ok(()),
    Some(active) if active == COMPILED => Ok(()),
    Some(active) => {
      static REPORTED: std::sync::Once = std::sync::Once::new();
      REPORTED.call_once(|| {
        log::error!(
          "the `tauri` crate was built for {COMPILED} but the active runtime uses {active}, so the \
           GTK APIs and the Linux menu integration are unavailable. This happens when a single \
           build enables both the `gtk3` and `gtk4` features, which cargo does whenever the \
           dependency graph contains runtime crates that disagree on the GTK version. Link a \
           single runtime crate on Linux - GTK 3 and GTK 4 cannot be initialized in the same \
           process anyway."
        );
      });
      Err(crate::Error::GtkVersionMismatch {
        expected: COMPILED,
        active,
      })
    }
  }
}
