// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::api::{
  file::read_string,
  path::{resolve_path, BaseDirectory},
};
use serde::{Deserialize, Serialize};
use std::{
  fs::File,
  io::Write,
  path::{Path, PathBuf},
};

/// Tauri Settings.
#[derive(Default, Deserialize, Serialize)]
pub struct Settings {
  /// Whether the user allows notifications or not.
  #[cfg(notification_all)]
  pub allow_notification: Option<bool>,
}

/// Gets the path to the settings file
fn get_settings_path() -> crate::api::Result<PathBuf> {
  resolve_path(".tauri-settings.json", Some(BaseDirectory::App))
}

/// Write the settings to the file system.
#[allow(dead_code)]
pub(crate) fn write_settings(settings: Settings) -> crate::Result<()> {
  let settings_path = get_settings_path()?;
  let settings_folder = Path::new(&settings_path).parent().unwrap();
  if !settings_folder.exists() {
    std::fs::create_dir(settings_folder)?;
  }
  File::create(settings_path)
    .map_err(Into::into)
    .and_then(|mut f| {
      f.write_all(serde_json::to_string(&settings)?.as_bytes())
        .map_err(Into::into)
    })
}

/// Reads the settings from the file system.
pub fn read_settings() -> crate::Result<Settings> {
  let settings_path = get_settings_path()?;
  if settings_path.exists() {
    read_string(settings_path)
      .and_then(|settings| serde_json::from_str(settings.as_str()).map_err(Into::into))
      .map_err(Into::into)
  } else {
    Ok(Default::default())
  }
}
