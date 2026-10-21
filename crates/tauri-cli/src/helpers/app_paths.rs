// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::current_dir,
  ffi::OsStr,
  path::{Path, PathBuf},
  sync::OnceLock,
};

use ignore::{WalkBuilder, gitignore::GitignoreBuilder};

use tauri_utils::{
  config::parse::{ConfigFormat, folder_has_configuration_file, is_configuration_file},
  platform::Target,
};

const TAURI_GITIGNORE: &[u8] = include_bytes!("../../tauri.gitignore");
// path to the Tauri app (Rust crate) directory, usually `<project>/src-tauri/`
const ENV_TAURI_APP_PATH: &str = "TAURI_APP_PATH";
// path to the frontend app directory, usually `<project>/`
const ENV_TAURI_FRONTEND_PATH: &str = "TAURI_FRONTEND_PATH";

pub struct Dirs {
  pub tauri: &'static Path,
  pub frontend: &'static Path,
}

static FRONTEND_DIR: OnceLock<PathBuf> = OnceLock::new();
static TAURI_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn walk_builder(path: &Path) -> WalkBuilder {
  let mut builder = WalkBuilder::new(path);
  builder.add_custom_ignore_filename(".taurignore");
  builder.git_global(false);
  builder.parents(false);
  skip_ignored_entries(&mut builder, path, TAURI_GITIGNORE);
  builder
}

/// Skips the walked entries matching the given `.gitignore`-style rules.
///
/// The rules are matched in memory instead of being written to a temporary ignore file,
/// which could be tampered with by other users.
pub fn skip_ignored_entries(builder: &mut WalkBuilder, root: &Path, rules: &[u8]) {
  let mut gitignore = GitignoreBuilder::new(root);
  for line in String::from_utf8_lossy(rules).lines() {
    let _ = gitignore.add_line(None, line);
  }
  if let Ok(gitignore) = gitignore.build() {
    builder.filter_entry(move |entry| {
      let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
      !gitignore.matched(entry.path(), is_dir).is_ignore()
    });
  }
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
    // entries with an extension (files) first
    .sort_by_file_path(|a, b| {
      a.extension()
        .is_none()
        .cmp(&b.extension().is_none())
        .then_with(|| a.cmp(b))
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
  let p = PathBuf::from(std::env::var_os(ENV_TAURI_APP_PATH)?);
  dunce::canonicalize(p).ok()
}

fn env_tauri_frontend_path() -> Option<PathBuf> {
  let p = PathBuf::from(std::env::var_os(ENV_TAURI_FRONTEND_PATH)?);
  dunce::canonicalize(p).ok()
}

pub fn resolve_tauri_dir() -> Option<PathBuf> {
  let src_dir = env_tauri_app_path().or_else(|| current_dir().ok())?;

  for standard_tauri_path in [src_dir.clone(), src_dir.join("src-tauri")] {
    if standard_tauri_path
      .join(ConfigFormat::Json.into_file_name())
      .exists()
      || standard_tauri_path
        .join(ConfigFormat::Json5.into_file_name())
        .exists()
      || standard_tauri_path
        .join(ConfigFormat::Toml.into_file_name())
        .exists()
    {
      log::debug!(
        "Found Tauri project inside {} on early lookup",
        standard_tauri_path.display()
      );
      return Some(standard_tauri_path);
    }
  }

  log::debug!("resolving Tauri directory from {}", src_dir.display());

  lookup(&src_dir, |path| {
    folder_has_configuration_file(Target::Linux, path) || is_configuration_file(Target::Linux, path)
  })
  .map(|p| {
    if p.is_dir() {
      log::debug!("Found Tauri project directory {}", p.display());
      p
    } else {
      log::debug!("Found Tauri project configuration file {}", p.display());
      p.parent().unwrap().to_path_buf()
    }
  })
}

pub fn resolve_dirs() -> Dirs {
  let tauri = TAURI_DIR.get_or_init(|| resolve_tauri_dir().unwrap_or_else(|| {
    let env_var_name = env_tauri_app_path().is_some().then(|| format!("`{ENV_TAURI_APP_PATH}`"));
    panic!("Couldn't recognize the {} folder as a Tauri project. It must contain a `{}`, `{}` or `{}` file in any subfolder.",
      env_var_name.as_deref().unwrap_or("current"),
      ConfigFormat::Json.into_file_name(),
      ConfigFormat::Json5.into_file_name(),
      ConfigFormat::Toml.into_file_name()
    )
  }));
  let frontend = FRONTEND_DIR.get_or_init(|| {
    resolve_frontend_dir().unwrap_or_else(|| tauri.parent().unwrap().to_path_buf())
  });
  Dirs { tauri, frontend }
}

pub fn resolve_frontend_dir() -> Option<PathBuf> {
  let frontend_dir =
    env_tauri_frontend_path().unwrap_or_else(|| current_dir().expect("failed to read cwd"));

  if frontend_dir.join("package.json").exists() {
    return Some(frontend_dir);
  }

  log::debug!(
    "resolving frontend directory from {}",
    frontend_dir.display()
  );

  lookup(&frontend_dir, |path| {
    if let Some(file_name) = path.file_name() {
      file_name == OsStr::new("package.json")
    } else {
      false
    }
  })
  .map(|p| p.parent().unwrap().to_path_buf())
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, fs, path::PathBuf};

  #[test]
  fn walk_builder_skips_default_ignored_entries() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for path in [
      "node_modules/pkg/package.json",
      "target/debug/app",
      "src/main.rs",
    ] {
      let path = root.join(path);
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, "").unwrap();
    }

    let paths = super::walk_builder(root)
      .require_git(false)
      .build()
      .flatten()
      .map(|entry| entry.path().strip_prefix(root).unwrap().to_path_buf())
      .collect::<Vec<_>>();

    assert!(paths.contains(&PathBuf::from("src/main.rs")), "{paths:?}");
    assert!(
      !paths
        .iter()
        .any(|p| p.starts_with("node_modules") || p.starts_with("target")),
      "{paths:?}"
    );
  }

  #[test]
  fn lookup_visits_files_first() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("a/b")).unwrap();
    fs::write(root.join("a/b/tauri.conf.json"), "").unwrap();
    fs::write(root.join("z.json"), "").unwrap();
    fs::write(root.join("c.json"), "").unwrap();

    let visited = RefCell::new(Vec::new());
    super::lookup(root, |path| {
      visited
        .borrow_mut()
        .push(path.strip_prefix(root).unwrap().to_path_buf());
      false
    });
    let visited = visited.into_inner();

    // the walk root comes first, then the files of each directory before its subdirectories
    assert_eq!(
      visited,
      vec![
        PathBuf::from(""),
        PathBuf::from("c.json"),
        PathBuf::from("z.json"),
        PathBuf::from("a"),
        PathBuf::from("a/b"),
        PathBuf::from("a/b/tauri.conf.json"),
      ]
    );
  }
}
