// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app;
pub mod dmg;
pub mod icon;
#[cfg(target_os = "macos")]
pub mod ios;
#[cfg(target_os = "macos")]
pub mod sign;

/// A path that has to be signed, in the order required by Apple.
///
/// Only the macOS signing path consumes these, but the bundle layout code that
/// collects them is shared with the Linux cross-compilation path.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct SignTarget {
  pub path: std::path::PathBuf,
  pub is_an_executable: bool,
}
