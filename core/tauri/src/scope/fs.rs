// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt, path::Path};

use glob::Pattern;
use tauri_utils::{
  config::{Config, FsAllowlistScope},
  Env, PackageInfo,
};

use crate::api::path::parse as parse_path;

/// Scope for filesystem access.
#[derive(Clone)]
pub struct Scope {
  allow_patterns: Vec<Pattern>,
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Scope")
      .field(
        "allow_patterns",
        &self
          .allow_patterns
          .iter()
          .map(|p| p.as_str())
          .collect::<Vec<&str>>(),
      )
      .finish()
  }
}

impl Scope {
  /// Creates a new scope from a `FsAllowlistScope` configuration.
  pub fn for_fs_api(
    config: &Config,
    package_info: &PackageInfo,
    env: &Env,
    scope: &FsAllowlistScope,
  ) -> Self {
    let mut allow_patterns = Vec::new();
    for path in &scope.0 {
      if let Ok(path) = parse_path(config, package_info, env, path) {
        allow_patterns.push(Pattern::new(&path.to_string_lossy()).expect("invalid glob pattern"));
        #[cfg(windows)]
        {
          allow_patterns.push(
            Pattern::new(&format!("\\\\?\\{}", path.display())).expect("invalid glob pattern"),
          );
        }
      }
    }
    Self { allow_patterns }
  }

  /// Determines if the given path is allowed on this scope.
  pub fn is_allowed<P: AsRef<Path>>(&self, path: P) -> bool {
    let path = path.as_ref();
    let path = if !path.exists() {
      crate::Result::Ok(path.to_path_buf())
    } else {
      std::fs::canonicalize(path).map_err(Into::into)
    };

    if let Ok(path) = path {
      let allowed = self.allow_patterns.iter().any(|p| p.matches_path(&path));
      allowed
    } else {
      false
    }
  }
}
