// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runtime access scopes.
//!
//! A scope narrows what an allowed feature may actually touch at runtime.
//! Tauri has two complementary scoping mechanisms:
//!
//! - **The IPC scope**, which is part of the [Access Control List](crate::ipc::RuntimeAuthority):
//!   capabilities define which commands each window/webview and origin may call, and the scope
//!   attached to a permission is read from a command with [`CommandScope`](crate::ipc::CommandScope)
//!   and [`GlobalScope`](crate::ipc::GlobalScope). That scope is static - it is resolved from the
//!   capability files at compile time (or added at runtime with
//!   [`Manager::add_capability`](crate::Manager::add_capability)) - and it is the command
//!   implementation that decides what to do with it.
//! - **The file system scope** in [`mod@fs`], a mutable list of allowed and forbidden glob patterns
//!   that the [asset protocol](https://v2.tauri.app/security/asset-protocol/) checks before serving
//!   a file. It is seeded from `app > security > assetProtocol > scope` in the configuration and
//!   can be changed while the application is running.
//!
//! Use `Manager::asset_protocol_scope` (`protocol-asset` Cargo feature) to get the asset
//! protocol scope and [`fs::Scope::allow_directory`], [`fs::Scope::allow_file`],
//! [`fs::Scope::forbid_directory`] and [`fs::Scope::forbid_file`] to change it, for instance to
//! grant access to a file the user just picked in a dialog:
//!
//! ```rust
//! # #[cfg(feature = "protocol-asset")]
//! # fn run() {
//! use tauri::Manager;
//!
//! tauri::Builder::default()
//!   .setup(|app| {
//!     app.asset_protocol_scope().allow_directory("/home/user/pictures", true)?;
//!     Ok(())
//!   });
//! # }
//! ```
//!
//! Scope changes are not persisted: they are lost when the application restarts.
//! Use the [persisted scope plugin](https://v2.tauri.app/plugin/persisted-scope/)
//! if you need them to survive a restart.

/// FS scope.
pub mod fs;

use std::path::Path;

/// Unique id of a scope event.
pub type ScopeEventId = u32;

/// Managed state for all the core scopes in a tauri application.
pub struct Scopes {
  #[cfg(feature = "protocol-asset")]
  pub(crate) asset_protocol: fs::Scope,
}

#[allow(unused)]
impl Scopes {
  /// Allows a directory on the scopes.
  pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.allow_directory(path, recursive)?;
    Ok(())
  }

  /// Allows a file on the scopes.
  pub fn allow_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.allow_file(path)?;
    Ok(())
  }

  /// Forbids a file on the scopes.
  pub fn forbid_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.forbid_file(path)?;
    Ok(())
  }
}
