// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
/// IPC scope.
pub mod ipc;

pub use self::ipc::Scope as IpcScope;
pub use fs::{Event as FsScopeEvent, Pattern as GlobPattern, Scope as FsScope};
use std::path::Path;

/// Unique id of a scope event.
pub type ScopeEventId = u32;

/// Managed state for all the core scopes in a tauri application.
pub struct Scopes {
  pub(crate) ipc: IpcScope,
  #[cfg(feature = "protocol-asset")]
  pub(crate) asset_protocol: FsScope,
}

impl Scopes {
  /// Allows a directory on the scopes.
  #[allow(unused)]
  pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.allow_directory(path, recursive)?;
    Ok(())
  }

  /// Allows a file on the scopes.
  #[allow(unused)]
  pub fn allow_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.allow_file(path)?;
    Ok(())
  }

  /// Forbids a file on the scopes.
  #[allow(unused)]
  pub fn forbid_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    #[cfg(feature = "protocol-asset")]
    self.asset_protocol.forbid_file(path)?;
    Ok(())
  }
}
