// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::{
  fs,
  path::{Path, PathBuf},
};

use tauri_utils::display_path;

use crate::{
  Result,
  error::{Context, ErrorExt},
};

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

/// `build.target` can be a single target or a list of targets.
#[derive(Deserialize)]
#[serde(untagged)]
enum BuildTarget {
  Single(String),
  Multiple(Vec<String>),
}

impl BuildTarget {
  fn into_target(self) -> Option<String> {
    match self {
      Self::Single(target) => Some(target),
      Self::Multiple(targets) => {
        if targets.len() > 1 {
          log::warn!(
            "Multiple targets configured in `build.target` ({}), using the first one: {}",
            targets.join(", "),
            targets[0]
          );
        }
        targets.into_iter().next()
      }
    }
  }
}

#[derive(Deserialize)]
struct BuildConfigSchema {
  target: Option<BuildTarget>,
}

#[derive(Default)]
pub struct BuildConfig {
  target: Option<String>,
}

#[derive(Deserialize)]
struct ConfigSchema {
  build: Option<BuildConfigSchema>,
}

impl ConfigSchema {
  fn build_target(self) -> Option<String> {
    self.build?.target?.into_target()
  }
}

#[derive(Default)]
pub struct Config {
  build: BuildConfig,
}

impl Config {
  pub fn load(path: &Path) -> Result<Self> {
    let mut config = Self::default();

    if let Ok(target) = std::env::var("CARGO_BUILD_TARGET")
      && !target.is_empty()
    {
      config.build.target = Some(target);
      return Ok(config);
    }

    let get_config = |path: PathBuf| -> Result<ConfigSchema> {
      let contents =
        fs::read_to_string(&path).fs_context("failed to read configuration file", path.clone())?;
      toml::from_str(&contents).context(format!(
        "could not parse TOML configuration in `{}`",
        display_path(&path)
      ))
    };

    // the environment variable takes precedence over the configuration files
    if let Ok(target) = std::env::var("CARGO_BUILD_TARGET") {
      let targets = target
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
      config.build.target = BuildTarget::Multiple(targets).into_target();
    }

    if config.build.target.is_none() {
      for current in PathAncestors::new(path) {
        if let Some(path) = get_file_path(&current.join(".cargo"), "config", true)? {
          if let Some(target) = get_config(path)?.build_target() {
            config.build.target = Some(target);
            break;
          }
        }
      }
    }

    if config.build.target.is_none()
      && let Ok(cargo_home) = std::env::var("CARGO_HOME")
      && let Some(path) = get_file_path(&PathBuf::from(cargo_home), "config", true)?
    {
      let toml = get_config(path)?;
      if let Some(target) = toml.build_target() {
        config.build.target = Some(target);
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

#[cfg(test)]
mod tests {
  use super::ConfigSchema;

  fn build_target(toml: &str) -> Option<String> {
    toml::from_str::<ConfigSchema>(toml).unwrap().build_target()
  }

  #[test]
  fn parse_build_target() {
    assert_eq!(build_target(""), None);
    assert_eq!(build_target("[build]\njobs = 2"), None);
    assert_eq!(
      build_target("[build]\ntarget = \"x86_64-pc-windows-msvc\""),
      Some("x86_64-pc-windows-msvc".into())
    );
    assert_eq!(
      build_target("[build]\ntarget = [\"aarch64-apple-darwin\"]"),
      Some("aarch64-apple-darwin".into())
    );
    assert_eq!(
      build_target("[build]\ntarget = [\"aarch64-apple-darwin\", \"x86_64-apple-darwin\"]"),
      Some("aarch64-apple-darwin".into())
    );
    assert_eq!(build_target("[build]\ntarget = []"), None);
  }
}
