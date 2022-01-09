// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
mod shell;

pub use fs::Scope as FsScope;
pub use shell::Scope as ShellScope;

pub(crate) struct Scopes {
  pub fs: FsScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
}
