// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
mod http;

pub use self::http::Scope as HttpScope;
pub use fs::Scope as FsScope;

pub(crate) struct Scopes {
  pub fs: FsScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
  #[cfg(http_request)]
  pub http: HttpScope,
  #[cfg(shell_execute)]
  pub shell: FsScope,
}
