// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{env::current_dir, path::PathBuf};

use once_cell::sync::Lazy;

fn get_tauri_dir() -> PathBuf {
  glob::glob(
    &current_dir()
      .expect("failed to read cwd")
      .join("**/tauri.conf.json")
      .to_string_lossy()
      .into_owned(),
  )
  .unwrap()
  .filter_map(Result::ok)
  .last()
  .map(|p| p.parent().unwrap().to_path_buf())
  .expect("Couldn't recognize the current folder as a Tauri project. It must contain a `tauri.conf.json` file in any subfolder.")
}

fn get_app_dir() -> Option<PathBuf> {
  glob::glob(
    &current_dir()
      .expect("failed to read cwd")
      .join("**/package.json")
      .to_string_lossy()
      .into_owned(),
  )
  .unwrap()
  .filter_map(Result::ok)
  .filter(|p| !p.to_string_lossy().into_owned().contains("node_modules"))
  .last()
  .map(|p| p.parent().unwrap().to_path_buf())
}

pub fn app_dir() -> &'static PathBuf {
  static APP_DIR: Lazy<PathBuf> =
    Lazy::new(|| get_app_dir().unwrap_or_else(|| get_tauri_dir().parent().unwrap().to_path_buf()));
  &APP_DIR
}

pub fn tauri_dir() -> PathBuf {
  get_tauri_dir()
}
