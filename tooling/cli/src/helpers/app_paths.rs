// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::current_dir,
  ffi::OsStr,
  path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use once_cell::sync::Lazy;

const TAURI_GITIGNORE: &[u8] = include_bytes!("../../tauri.gitignore");

fn lookup<F: Fn(&PathBuf) -> bool>(dir: &Path, checker: F) -> Option<PathBuf> {
  let mut default_gitignore = std::env::temp_dir();
  default_gitignore.push(".gitignore");
  if !default_gitignore.exists() {
    if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
      use std::io::Write;
      let _ = file.write_all(TAURI_GITIGNORE);
    }
  }

  let mut builder = WalkBuilder::new(dir);
  let _ = builder.add_ignore(default_gitignore);
  builder.require_git(false).ignore(false).max_depth(Some(2));

  for entry in builder.build().flatten() {
    let path = dir.join(entry.path());
    if checker(&path) {
      return Some(path);
    }
  }
  None
}

fn get_tauri_dir() -> PathBuf {
  lookup(&current_dir().expect("failed to read cwd"), |path| if let Some(file_name) = path.file_name() {
    file_name == OsStr::new("tauri.conf.json") || file_name == OsStr::new("tauri.conf.json5")
  } else {
    false
  })
  .map(|p| p.parent().unwrap().to_path_buf())
  .expect("Couldn't recognize the current folder as a Tauri project. It must contain a `tauri.conf.json` or `tauri.conf.json5` file in any subfolder.")
}

fn get_app_dir() -> Option<PathBuf> {
  lookup(&current_dir().expect("failed to read cwd"), |path| {
    if let Some(file_name) = path.file_name() {
      file_name == OsStr::new("package.json")
    } else {
      false
    }
  })
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
