// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
/// IPC scope.
pub mod ipc;

pub use self::ipc::Scope as IpcScope;
pub use fs::{Event as FsScopeEvent, Pattern as GlobPattern, Scope as FsScope};

pub(crate) struct Scopes {
  pub ipc: IpcScope,
  #[cfg(feature = "protocol-asset")]
  pub asset_protocol: FsScope,
}
