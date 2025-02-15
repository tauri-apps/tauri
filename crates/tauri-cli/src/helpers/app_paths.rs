// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  cmp::Ordering,
  env::current_dir,
  ffi::OsStr,
  path::{Path, PathBuf},
  sync::OnceLock,
};

use ignore::WalkBuilder;

use tauri_utils::{
  config::parse::{folder_has_configuration_file, is_configuration_file, ConfigFormat},
  platform::Target,
};

const TAURI_GITIGNORE: &[u8] = include_bytes!("../../tauri.gitignore");
// path to the Tauri app (Rust crate) directory, usually `<project>/src-tauri/`
const ENV_TAURI_APP_PATH: &str = "TAURI_APP_PATH";
// path to the frontend app directory, usually `<project>/`
const ENV_TAURI_FRONTEND_PATH: &str = "TAURI_FRONTEND_PATH";

static FRONTEND_DIR: OnceLock<PathBuf> = OnceLock::new();
static TAURI_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn walk_builder(path: &Path) -> WalkBuilder {
  let mut default_gitignore = std::env::temp_dir();
  default_gitignore.push(".gitignore");
  if !default_gitignore.exists() {
    if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
      use std::io::Write;
      let _ = file.write_all(TAURI_GITIGNORE);
    }
  }

  let mut builder = WalkBuilder::new(path);
  builder.add_custom_ignore_filename(".taurignore");
  builder.git_global(false);
  let _ = builder.add_ignore(default_gitignore);
  builder
}

fn lookup<F: Fn(&PathBuf) -> bool>(dir: &Path, checker: F) -> Option<PathBuf> {
  let mut builder = walk_builder(dir);
  builder
    .require_git(false)
    .ignore(false)
    .max_depth(Some(
      std::env::var("TAURI_CLI_CONFIG_DEPTH")
        .map(|d| {
          d.parse()
            .expect("`TAURI_CLI_CONFIG_DEPTH` environment variable must be a positive integer")
        })
        .unwrap_or(3),
    ))
    .sort_by_file_path(|a, _| {
      if a.extension().is_some() {
        Ordering::Less
      } else {
        Ordering::Greater
      }
    });

  for entry in builder.build().flatten() {
    let path = dir.join(entry.path());
    if checker(&path) {
      return Some(path);
    }
  }
  None
}

fn env_tauri_app_path() -> Option<PathBuf> {
  std::env::var(ENV_TAURI_APP_PATH)
    .map(PathBuf::from)
    .ok()?
    .canonicalize()
    .ok()
    .map(|p| dunce::simplified(&p).to_path_buf())
}

fn env_tauri_frontend_path() -> Option<PathBuf> {
  std::env::var(ENV_TAURI_FRONTEND_PATH)
    .map(PathBuf::from)
    .ok()?
    .canonicalize()
    .ok()
    .map(|p| dunce::simplified(&p).to_path_buf())
}

pub fn resolve_tauri_dir() -> Option<PathBuf> {
  let src_dir = env_tauri_app_path().or_else(|| current_dir().ok())?;

  if src_dir.join(ConfigFormat::Json.into_file_name()).exists()
    || src_dir.join(ConfigFormat::Json5.into_file_name()).exists()
    || src_dir.join(ConfigFormat::Toml.into_file_name()).exists()
  {
    return Some(src_dir);
  }

  lookup(&src_dir, |path| {
    folder_has_configuration_file(Target::Linux, path) || is_configuration_file(Target::Linux, path)
  })
  .map(|p| {
    if p.is_dir() {
      p
    } else {
      p.parent().unwrap().to_path_buf()
    }
  })
}

pub fn resolve() {
  TAURI_DIR.set(resolve_tauri_dir().unwrap_or_else(|| {
    let env_var_name = env_tauri_app_path().is_some().then(|| format!("`{ENV_TAURI_APP_PATH}`"));
    panic!("Couldn't recognize the {} folder as a Tauri project. It must contain a `{}`, `{}` or `{}` file in any subfolder.",
      env_var_name.as_deref().unwrap_or("current"),
      ConfigFormat::Json.into_file_name(),
      ConfigFormat::Json5.into_file_name(),
      ConfigFormat::Toml.into_file_name()
    )
  })).expect("tauri dir already resolved");
  FRONTEND_DIR
    .set(resolve_frontend_dir().unwrap_or_else(|| tauri_dir().parent().unwrap().to_path_buf()))
    .expect("app dir already resolved");
}

pub fn tauri_dir() -> &'static PathBuf {
  TAURI_DIR
    .get()
    .expect("app paths not initialized, this is a Tauri CLI bug")
}

pub fn resolve_frontend_dir() -> Option<PathBuf> {
  let frontend_dir =
    env_tauri_frontend_path().unwrap_or_else(|| current_dir().expect("failed to read cwd"));

  if frontend_dir.join("package.json").exists() {
    return Some(frontend_dir);
  }

  lookup(&frontend_dir, |path| {
    if let Some(file_name) = path.file_name() {
      file_name == OsStr::new("package.json")
    } else {
      false
    }
  })
  .map(|p| p.parent().unwrap().to_path_buf())
}

pub fn frontend_dir() -> &'static PathBuf {
  FRONTEND_DIR
    .get()
    .expect("app paths not initialized, this is a Tauri CLI bug")
}
