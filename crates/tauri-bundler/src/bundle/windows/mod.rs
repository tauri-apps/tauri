// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "windows")]
pub mod msi;
pub mod nsis;
pub mod sign;

mod util;
pub use util::{
  NSIS_OUTPUT_FOLDER_NAME, NSIS_UPDATER_OUTPUT_FOLDER_NAME, WIX_OUTPUT_FOLDER_NAME,
  WIX_UPDATER_OUTPUT_FOLDER_NAME,
};
