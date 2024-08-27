// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
