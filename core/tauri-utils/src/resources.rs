// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Component, Path, PathBuf};

/// Given a path (absolute or relative) to a resource file, returns the
/// relative path from the bundle resources directory where that resource
/// should be stored.
pub fn resource_relpath(path: &Path) -> PathBuf {
  let mut dest = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Prefix(_) => {}
      Component::RootDir => dest.push("_root_"),
      Component::CurDir => {}
      Component::ParentDir => dest.push("_up_"),
      Component::Normal(string) => dest.push(string),
    }
  }
  dest
}

/// Parses the external binaries to bundle, adding the target triple suffix to each of them.
pub fn external_binaries(external_binaries: &[String], target_triple: &str) -> Vec<String> {
  let mut paths = Vec::new();
  for curr_path in external_binaries {
    paths.push(format!(
      "{}-{}{}",
      curr_path,
      target_triple,
      if cfg!(windows) { ".exe" } else { "" }
    ));
  }
  paths
}

/// A helper to iterate through resources.
pub struct ResourcePaths<'a> {
  /// the patterns to iterate.
  pattern_iter: std::slice::Iter<'a, String>,
  /// the glob iterator if the path from the current iteration is a glob pattern.
  glob_iter: Option<glob::Paths>,
  /// the walkdir iterator if the path from the current iteration is a directory.
  walk_iter: Option<walkdir::IntoIter>,
  /// whether the resource paths allows directories or not.
  allow_walk: bool,
  /// the pattern of the current iteration.
  current_pattern: Option<String>,
  /// whether the current pattern is valid or not.
  current_pattern_is_valid: bool,
}

impl<'a> ResourcePaths<'a> {
  /// Creates a new ResourcePaths from a slice of patterns to iterate
  pub fn new(patterns: &'a [String], allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      pattern_iter: patterns.iter(),
      glob_iter: None,
      walk_iter: None,
      allow_walk,
      current_pattern: None,
      current_pattern_is_valid: false,
    }
  }
}

impl<'a> Iterator for ResourcePaths<'a> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    loop {
      if let Some(ref mut walk_entries) = self.walk_iter {
        if let Some(entry) = walk_entries.next() {
          let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(Err(crate::Error::from(error))),
          };
          let path = entry.path();
          if path.is_dir() {
            continue;
          }
          self.current_pattern_is_valid = true;
          return Some(Ok(path.to_path_buf()));
        }
      }
      self.walk_iter = None;
      if let Some(ref mut glob_paths) = self.glob_iter {
        if let Some(glob_result) = glob_paths.next() {
          let path = match glob_result {
            Ok(path) => path,
            Err(error) => return Some(Err(error.into())),
          };
          if path.is_dir() {
            if self.allow_walk {
              let walk = walkdir::WalkDir::new(path);
              self.walk_iter = Some(walk.into_iter());
              continue;
            } else {
              return Some(Err(crate::Error::NotAllowedToWalkDir(path)));
            }
          }
          self.current_pattern_is_valid = true;
          return Some(Ok(path));
        } else if let Some(current_path) = &self.current_pattern {
          if !self.current_pattern_is_valid {
            self.glob_iter = None;
            return Some(Err(crate::Error::GlobPathNotFound(current_path.clone())));
          }
        }
      }
      self.glob_iter = None;
      if let Some(pattern) = self.pattern_iter.next() {
        self.current_pattern = Some(pattern.to_string());
        self.current_pattern_is_valid = false;
        let glob = match glob::glob(pattern) {
          Ok(glob) => glob,
          Err(error) => return Some(Err(error.into())),
        };
        self.glob_iter = Some(glob);
        continue;
      }
      return None;
    }
  }
}
