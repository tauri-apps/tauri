// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
  fs,
  path::{Path, PathBuf},
};

use tauri_utils::display_path;

struct PathAncestors<'a> {
  current: Option<&'a Path>,
}

impl<'a> PathAncestors<'a> {
  fn new(path: &'a Path) -> PathAncestors<'a> {
    PathAncestors {
      current: Some(path),
    }
  }
}

impl<'a> Iterator for PathAncestors<'a> {
  type Item = &'a Path;

  fn next(&mut self) -> Option<&'a Path> {
    if let Some(path) = self.current {
      self.current = path.parent();

      Some(path)
    } else {
      None
    }
  }
}

#[derive(Default, Deserialize)]
pub struct BuildConfig {
  target: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigSchema {
  build: Option<BuildConfig>,
}

#[derive(Default)]
pub struct Config {
  build: BuildConfig,
}

impl Config {
  pub fn load(path: &Path) -> Result<Self> {
    let mut config = Self::default();

    let get_config = |path: PathBuf| -> Result<ConfigSchema> {
      let contents = fs::read_to_string(&path).with_context(|| {
        format!(
          "failed to read configuration file `{}`",
          display_path(&path)
        )
      })?;
      toml::from_str(&contents).with_context(|| {
        format!(
          "could not parse TOML configuration in `{}`",
          display_path(&path)
        )
      })
    };

    for current in PathAncestors::new(path) {
      if let Some(path) = get_file_path(&current.join(".cargo"), "config", true)? {
        let toml = get_config(path)?;
        if let Some(target) = toml.build.and_then(|b| b.target) {
          config.build.target = Some(target);
          break;
        }
      }
    }

    if config.build.target.is_none() {
      if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        if let Some(path) = get_file_path(&PathBuf::from(cargo_home), "config", true)? {
          let toml = get_config(path)?;
          if let Some(target) = toml.build.and_then(|b| b.target) {
            config.build.target = Some(target);
          }
        }
      }
    }

    Ok(config)
  }

  pub fn build(&self) -> &BuildConfig {
    &self.build
  }
}

impl BuildConfig {
  pub fn target(&self) -> Option<&str> {
    self.target.as_deref()
  }
}

/// The purpose of this function is to aid in the transition to using
/// .toml extensions on Cargo's config files, which were historically not used.
/// Both 'config.toml' and 'credentials.toml' should be valid with or without extension.
/// When both exist, we want to prefer the one without an extension for
/// backwards compatibility, but warn the user appropriately.
fn get_file_path(
  dir: &Path,
  filename_without_extension: &str,
  warn: bool,
) -> Result<Option<PathBuf>> {
  let possible = dir.join(filename_without_extension);
  let possible_with_extension = dir.join(format!("{filename_without_extension}.toml"));

  if possible.exists() {
    if warn && possible_with_extension.exists() {
      // We don't want to print a warning if the version
      // without the extension is just a symlink to the version
      // WITH an extension, which people may want to do to
      // support multiple Cargo versions at once and not
      // get a warning.
      let skip_warning = if let Ok(target_path) = fs::read_link(&possible) {
        target_path == possible_with_extension
      } else {
        false
      };

      if !skip_warning {
        log::warn!(
          "Both `{}` and `{}` exist. Using `{}`",
          display_path(&possible),
          display_path(&possible_with_extension),
          display_path(&possible)
        );
      }
    }

    Ok(Some(possible))
  } else if possible_with_extension.exists() {
    Ok(Some(possible_with_extension))
  } else {
    Ok(None)
  }
}
