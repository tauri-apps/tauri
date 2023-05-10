// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
/// IPC scope.
pub mod ipc;

pub use self::ipc::Scope as IpcScope;
pub use fs::{Event as FsScopeEvent, Pattern as GlobPattern, Scope as FsScope};
use std::path::Path;

pub(crate) struct Scopes {
  pub ipc: IpcScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
}

impl Scopes {
  #[allow(dead_code)]
  pub(crate) fn allow_directory(&self, path: &Path, recursive: bool) -> crate::Result<()> {
    #[cfg(protocol_asset)]
    self.asset_protocol.allow_directory(path, recursive)?;
    Ok(())
  }

  #[allow(dead_code)]
  pub(crate) fn allow_file(&self, path: &Path) -> crate::Result<()> {
    #[cfg(protocol_asset)]
    self.asset_protocol.allow_file(path)?;
    Ok(())
  }
}
