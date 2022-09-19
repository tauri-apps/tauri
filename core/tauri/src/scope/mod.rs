// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
mod http;
#[cfg(shell_scope)]
mod shell;

pub use self::http::Scope as HttpScope;
pub use fs::{Event as FsScopeEvent, Pattern as GlobPattern, Scope as FsScope};
#[cfg(shell_scope)]
pub use shell::{
  ExecuteArgs, Scope as ShellScope, ScopeAllowedArg as ShellScopeAllowedArg,
  ScopeAllowedCommand as ShellScopeAllowedCommand, ScopeConfig as ShellScopeConfig,
  ScopeError as ShellScopeError,
};
use std::path::Path;

pub(crate) struct Scopes {
  pub fs: FsScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
  #[cfg(http_request)]
  pub http: HttpScope,
  #[cfg(shell_scope)]
  pub shell: ShellScope,
}

impl Scopes {
  #[allow(dead_code)]
  pub(crate) fn allow_directory(&self, path: &Path, recursive: bool) -> crate::Result<()> {
    let path = path.to_string_lossy();
    let escaped_path = glob::Pattern::escape(&path);

    self.fs.allow_directory(&escaped_path, recursive)?;
    #[cfg(protocol_asset)]
    self
      .asset_protocol
      .allow_directory(&escaped_path, recursive)?;
    Ok(())
  }

  #[allow(dead_code)]
  pub(crate) fn allow_file(&self, path: &Path) -> crate::Result<()> {
    let path = path.to_string_lossy();
    let escaped_path = glob::Pattern::escape(&path);
    self.fs.allow_file(&escaped_path)?;
    #[cfg(protocol_asset)]
    self.asset_protocol.allow_file(&escaped_path)?;
    Ok(())
  }
}
