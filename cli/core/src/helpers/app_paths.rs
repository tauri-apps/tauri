// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{env::current_dir, path::PathBuf};

use once_cell::sync::Lazy;

fn get_app_dir() -> PathBuf {
  let mut dir = current_dir().expect("failed to read cwd");

  let mut count = 0;

  // only go up three folders max
  while count <= 2 {
    let test_path = dir.join("src-tauri/tauri.conf.json");
    if test_path.exists() {
      return dir;
    }
    count += 1;
    match dir.parent() {
      Some(parent) => {
        dir = parent.to_path_buf();
      }
      None => break,
    }
  }

  panic!("Couldn't recognize the current folder as a Tauri project.")
}

pub fn app_dir() -> &'static PathBuf {
  static APP_DIR: Lazy<PathBuf> = Lazy::new(get_app_dir);
  &APP_DIR
}

pub fn tauri_dir() -> PathBuf {
  app_dir().join("src-tauri")
}
