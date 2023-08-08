// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::app_paths::{app_dir, tauri_dir},
  Result,
};

mod config;
mod frontend;
mod manifest;

pub fn command() -> Result<()> {
  let tauri_dir = tauri_dir();
  let app_dir = app_dir();

  config::migrate(&tauri_dir)?;
  manifest::migrate(&tauri_dir)?;
  frontend::migrate(app_dir, &tauri_dir)?;

  Ok(())
}
