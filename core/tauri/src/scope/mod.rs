// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
mod http;
#[cfg(shell_scope)]
mod shell;

pub use self::http::Scope as HttpScope;
pub use fs::Scope as FsScope;
#[cfg(shell_scope)]
pub use shell::{
  ExecuteArgs, Scope as ShellScope, ScopeAllowedArg as ShellScopeAllowedArg,
  ScopeAllowedCommand as ShellScopeAllowedCommand, ScopeConfig as ShellScopeConfig,
  ScopeError as ShellScopeError,
};

pub(crate) struct Scopes {
  pub fs: FsScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
  #[cfg(http_request)]
  pub http: HttpScope,
  #[cfg(shell_scope)]
  pub shell: ShellScope,
}
