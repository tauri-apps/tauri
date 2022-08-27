// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Settings;

use std::path::PathBuf;

/// Bundles the project.
/// Not implemented yet.
pub fn bundle_project(_settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  unimplemented!();
}
