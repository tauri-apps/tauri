// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  path::{Component, Path, PathBuf},
};

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
      if target_triple.contains("windows") {
        ".exe"
      } else {
        ""
      }
    ));
  }
  paths
}

enum PatternIter<'a> {
  Slice(std::slice::Iter<'a, String>),
  Map(std::collections::hash_map::Iter<'a, String, String>),
}

/// A helper to iterate through resources.
pub struct ResourcePaths<'a> {
  iter: ResourcePathsIter<'a>,
}

impl<'a> ResourcePaths<'a> {
  /// Creates a new ResourcePaths from a slice of patterns to iterate
  pub fn new(patterns: &'a [String], allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      iter: ResourcePathsIter {
        pattern_iter: PatternIter::Slice(patterns.iter()),
        glob_iter: None,
        walk_iter: None,
        allow_walk,
        current_pattern: None,
        current_pattern_is_valid: false,
        current_dest: None,
      },
    }
  }

  /// Creates a new ResourcePaths from a slice of patterns to iterate
  pub fn from_map(patterns: &'a HashMap<String, String>, allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      iter: ResourcePathsIter {
        pattern_iter: PatternIter::Map(patterns.iter()),
        glob_iter: None,
        walk_iter: None,
        allow_walk,
        current_pattern: None,
        current_pattern_is_valid: false,
        current_dest: None,
      },
    }
  }

  /// Returns the resource iterator that yields the source and target paths.
  /// Needed when using [`Self::from_map`].
  pub fn iter(self) -> ResourcePathsIter<'a> {
    self.iter
  }
}

/// Iterator of a [`ResourcePaths`].
pub struct ResourcePathsIter<'a> {
  /// the patterns to iterate.
  pattern_iter: PatternIter<'a>,
  /// the glob iterator if the path from the current iteration is a glob pattern.
  glob_iter: Option<glob::Paths>,
  /// the walkdir iterator if the path from the current iteration is a directory.
  walk_iter: Option<walkdir::IntoIter>,
  /// whether the resource paths allows directories or not.
  allow_walk: bool,
  /// the pattern of the current iteration.
  current_pattern: Option<(String, PathBuf)>,
  /// whether the current pattern is valid or not.
  current_pattern_is_valid: bool,
  /// Current destination path. Only set when the iterator comes from a Map.
  current_dest: Option<PathBuf>,
}

/// Information for a resource.
pub struct Resource {
  path: PathBuf,
  target: PathBuf,
}

impl Resource {
  /// The path of the resource.
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// The target location of the resource.
  pub fn target(&self) -> &Path {
    &self.target
  }
}

impl<'a> Iterator for ResourcePaths<'a> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    self.iter.next().map(|r| r.map(|res| res.path))
  }
}

fn normalize(path: &Path) -> PathBuf {
  let mut dest = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Prefix(_) => {}
      Component::RootDir => dest.push("/"),
      Component::CurDir => {}
      Component::ParentDir => dest.push(".."),
      Component::Normal(string) => dest.push(string),
    }
  }
  dest
}

impl<'a> Iterator for ResourcePathsIter<'a> {
  type Item = crate::Result<Resource>;

  fn next(&mut self) -> Option<crate::Result<Resource>> {
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
          return Some(Ok(Resource {
            target: if let (Some(current_dest), Some(current_pattern)) =
              (&self.current_dest, &self.current_pattern)
            {
              if current_pattern.0.contains('*') {
                current_dest.join(path.file_name().unwrap())
              } else {
                current_dest.join(path.strip_prefix(&current_pattern.1).unwrap())
              }
            } else {
              resource_relpath(path)
            },
            path: path.to_path_buf(),
          }));
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
          return Some(Ok(Resource {
            target: if let Some(current_dest) = &self.current_dest {
              current_dest.join(path.file_name().unwrap())
            } else {
              resource_relpath(&path)
            },
            path,
          }));
        } else if let Some(current_path) = &self.current_pattern {
          if !self.current_pattern_is_valid {
            self.glob_iter = None;
            return Some(Err(crate::Error::GlobPathNotFound(current_path.0.clone())));
          }
        }
      }
      self.glob_iter = None;
      self.current_dest = None;
      match &mut self.pattern_iter {
        PatternIter::Slice(iter) => {
          if let Some(pattern) = iter.next() {
            self.current_pattern = Some((pattern.to_string(), normalize(Path::new(pattern))));
            self.current_pattern_is_valid = false;
            let glob = match glob::glob(pattern) {
              Ok(glob) => glob,
              Err(error) => return Some(Err(error.into())),
            };
            self.glob_iter = Some(glob);
            continue;
          }
        }
        PatternIter::Map(iter) => {
          if let Some((pattern, dest)) = iter.next() {
            self.current_pattern = Some((pattern.to_string(), normalize(Path::new(pattern))));
            self.current_pattern_is_valid = false;
            let glob = match glob::glob(pattern) {
              Ok(glob) => glob,
              Err(error) => return Some(Err(error.into())),
            };
            self
              .current_dest
              .replace(resource_relpath(&PathBuf::from(dest)));
            self.glob_iter = Some(glob);
            continue;
          }
        }
      }
      return None;
    }
  }
}
