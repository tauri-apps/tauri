// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  path::{Component, Path, PathBuf},
};

use walkdir::WalkDir;

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

/// Information for a resource.
#[derive(Debug)]
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

#[derive(Debug)]
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
        allow_walk,
        current_path: None,
        current_pattern: None,
        current_dest: None,
        walk_iter: None,
        glob_iter: None,
      },
    }
  }

  /// Creates a new ResourcePaths from a slice of patterns to iterate
  pub fn from_map(patterns: &'a HashMap<String, String>, allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      iter: ResourcePathsIter {
        pattern_iter: PatternIter::Map(patterns.iter()),
        allow_walk,
        current_path: None,
        current_pattern: None,
        current_dest: None,
        walk_iter: None,
        glob_iter: None,
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
#[derive(Debug)]
pub struct ResourcePathsIter<'a> {
  /// the patterns to iterate.
  pattern_iter: PatternIter<'a>,
  /// whether the resource paths allows directories or not.
  allow_walk: bool,

  current_path: Option<PathBuf>,
  current_pattern: Option<String>,
  current_dest: Option<PathBuf>,

  walk_iter: Option<walkdir::IntoIter>,
  glob_iter: Option<glob::Paths>,
}

impl<'a> ResourcePathsIter<'a> {
  fn next_glob_iter(&mut self) -> Option<crate::Result<Resource>> {
    let entry = self.glob_iter.as_mut().unwrap().next()?;

    let entry = match entry {
      Ok(entry) => entry,
      Err(err) => return Some(Err(err.into())),
    };

    self.current_path = Some(normalize(&entry));
    self.next_current_path()
  }

  fn next_walk_iter(&mut self) -> Option<crate::Result<Resource>> {
    let entry = self.walk_iter.as_mut().unwrap().next()?;

    let entry = match entry {
      Ok(entry) => entry,
      Err(err) => return Some(Err(err.into())),
    };

    self.current_path = Some(normalize(entry.path()));
    self.next_current_path()
  }

  fn resource_from_path(&mut self, path: &Path) -> crate::Result<Resource> {
    if !path.exists() {
      return Err(crate::Error::ResourcePathNotFound(path.to_path_buf()));
    }

    Ok(Resource {
      path: path.to_path_buf(),
      target: self
        .current_dest
        .as_ref()
        .map(|current_dest| {
          // if processing a directory, preserve directory structure under current_dest
          if self.walk_iter.is_some() {
            let current_pattern = self.current_pattern.as_ref().unwrap();
            current_dest.join(path.strip_prefix(current_pattern).unwrap_or(path))
          } else if current_dest.components().count() == 0 {
            // if current_dest is empty while processing a file pattern or glob
            // we preserve the file name as it is
            PathBuf::from(path.file_name().unwrap())
          } else if self.glob_iter.is_some() {
            // if processing a glob and current_dest is not empty
            // we put all globbed paths under current_dest
            // preserving the file name as it is
            current_dest.join(path.file_name().unwrap())
          } else {
            current_dest.clone()
          }
        })
        .unwrap_or_else(|| resource_relpath(path)),
    })
  }

  fn next_current_path(&mut self) -> Option<crate::Result<Resource>> {
    // should be safe to unwrap since every call to `self.next_current_path()`
    // is preceeded with assignemt to `self.current_path`
    let path = self.current_path.take().unwrap();

    let is_dir = path.is_dir();

    if is_dir {
      if self.glob_iter.is_some() {
        return self.next();
      }

      if !self.allow_walk {
        return Some(Err(crate::Error::NotAllowedToWalkDir(path.to_path_buf())));
      }

      if self.walk_iter.is_none() {
        self.walk_iter = Some(WalkDir::new(&path).into_iter());
      }

      match self.next_walk_iter() {
        Some(resource) => Some(resource),
        None => {
          self.walk_iter = None;
          self.next()
        }
      }
    } else {
      Some(self.resource_from_path(&path))
    }
  }

  fn next_pattern(&mut self) -> Option<crate::Result<Resource>> {
    self.current_pattern = None;
    self.current_dest = None;
    self.current_path = None;

    let pattern = match &mut self.pattern_iter {
      PatternIter::Slice(iter) => match iter.next() {
        Some(pattern) => pattern,
        None => return None,
      },
      PatternIter::Map(iter) => match iter.next() {
        Some((pattern, dest)) => {
          self.current_pattern = Some(pattern.clone());
          self.current_dest = Some(resource_relpath(Path::new(dest)));
          pattern
        }
        None => return None,
      },
    };

    if pattern.contains('*') {
      self.glob_iter = match glob::glob(pattern) {
        Ok(glob) => Some(glob),
        Err(error) => return Some(Err(error.into())),
      };
      match self.next_glob_iter() {
        Some(r) => return Some(r),
        None => {
          self.glob_iter = None;
          return Some(Err(crate::Error::GlobPathNotFound(pattern.clone())));
        }
      }
    }

    self.current_path = Some(normalize(Path::new(pattern)));
    self.next_current_path()
  }
}

impl<'a> Iterator for ResourcePaths<'a> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    self.iter.next().map(|r| r.map(|res| res.path))
  }
}

impl<'a> Iterator for ResourcePathsIter<'a> {
  type Item = crate::Result<Resource>;

  fn next(&mut self) -> Option<crate::Result<Resource>> {
    if self.current_path.is_some() {
      return self.next_current_path();
    }

    if self.walk_iter.is_some() {
      match self.next_walk_iter() {
        Some(r) => return Some(r),
        None => self.walk_iter = None,
      }
    }

    if self.glob_iter.is_some() {
      match self.next_glob_iter() {
        Some(r) => return Some(r),
        None => self.glob_iter = None,
      }
    }

    self.next_pattern()
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use std::fs;
  use std::path::Path;

  impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
      self.path == other.path && self.target == other.target
    }
  }

  fn expected_resources(resources: &[(&str, &str)]) -> Vec<Resource> {
    resources
      .iter()
      .map(|(path, target)| Resource {
        path: Path::new(path).components().collect(),
        target: Path::new(target).components().collect(),
      })
      .collect()
  }

  fn setup_test_dirs() {
    let mut random = [0; 1];
    getrandom::getrandom(&mut random).unwrap();

    let temp = std::env::temp_dir();
    let temp = temp.join(format!("tauri_resource_paths_iter_test_{}", random[0]));

    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();

    std::env::set_current_dir(&temp).unwrap();

    let paths = [
      Path::new("src-tauri/tauri.conf.json"),
      Path::new("src-tauri/some-other-json.json"),
      Path::new("src-tauri/Cargo.toml"),
      Path::new("src-tauri/Tauri.toml"),
      Path::new("src-tauri/build.rs"),
      Path::new("src/assets/javascript.svg"),
      Path::new("src/assets/tauri.svg"),
      Path::new("src/assets/rust.svg"),
      Path::new("src/assets/lang/en.json"),
      Path::new("src/assets/lang/ar.json"),
      Path::new("src/sounds/lang/es.wav"),
      Path::new("src/sounds/lang/fr.wav"),
      Path::new("src/textures/ground/earth.tex"),
      Path::new("src/textures/ground/sand.tex"),
      Path::new("src/textures/water.tex"),
      Path::new("src/textures/fire.tex"),
      Path::new("src/tiles/sky/grey.tile"),
      Path::new("src/tiles/sky/yellow.tile"),
      Path::new("src/tiles/grass.tile"),
      Path::new("src/tiles/stones.tile"),
      Path::new("src/index.html"),
      Path::new("src/style.css"),
      Path::new("src/script.js"),
      Path::new("src/dir/another-dir/file1.txt"),
      Path::new("src/dir/another-dir2/file2.txt"),
    ];

    for path in paths {
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, "").unwrap();
    }
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_slice_allow_walk() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::new(
      &[
        "../src/script.js".into(),
        "../src/assets".into(),
        "../src/index.html".into(),
        "../src/sounds".into(),
        "*.toml".into(),
        "*.conf.json".into(),
      ],
      true,
    )
    .iter()
    .flatten()
    .collect::<Vec<_>>();

    let expected = expected_resources(&[
      ("../src/script.js", "_up_/src/script.js"),
      (
        "../src/assets/javascript.svg",
        "_up_/src/assets/javascript.svg",
      ),
      ("../src/assets/tauri.svg", "_up_/src/assets/tauri.svg"),
      ("../src/assets/rust.svg", "_up_/src/assets/rust.svg"),
      ("../src/assets/lang/en.json", "_up_/src/assets/lang/en.json"),
      ("../src/assets/lang/ar.json", "_up_/src/assets/lang/ar.json"),
      ("../src/index.html", "_up_/src/index.html"),
      ("../src/sounds/lang/es.wav", "_up_/src/sounds/lang/es.wav"),
      ("../src/sounds/lang/fr.wav", "_up_/src/sounds/lang/fr.wav"),
      ("Cargo.toml", "Cargo.toml"),
      ("Tauri.toml", "Tauri.toml"),
      ("tauri.conf.json", "tauri.conf.json"),
    ]);

    assert_eq!(resources.len(), expected.len());
    for resource in expected {
      if !resources.contains(&resource) {
        panic!("{resource:?} was expected but not found in {resources:?}");
      }
    }
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_slice_no_walk() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::new(
      &[
        "../src/script.js".into(),
        "../src/assets".into(),
        "../src/index.html".into(),
        "../src/sounds".into(),
        "*.toml".into(),
        "*.conf.json".into(),
      ],
      false,
    )
    .iter()
    .flatten()
    .collect::<Vec<_>>();

    let expected = expected_resources(&[
      ("../src/script.js", "_up_/src/script.js"),
      ("../src/index.html", "_up_/src/index.html"),
      ("Cargo.toml", "Cargo.toml"),
      ("Tauri.toml", "Tauri.toml"),
      ("tauri.conf.json", "tauri.conf.json"),
    ]);

    assert_eq!(resources.len(), expected.len());
    for resource in expected {
      if !resources.contains(&resource) {
        panic!("{resource:?} was expected but not found in {resources:?}");
      }
    }
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_map_allow_walk() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::from_map(
      &std::collections::HashMap::from_iter([
        ("../src/script.js".into(), "main.js".into()),
        ("../src/assets".into(), "".into()),
        ("../src/index.html".into(), "frontend/index.html".into()),
        ("../src/sounds".into(), "voices".into()),
        ("../src/textures/*".into(), "textures".into()),
        ("../src/tiles/**/*".into(), "tiles".into()),
        ("*.toml".into(), "".into()),
        ("*.conf.json".into(), "json".into()),
        ("../non-existent-file".into(), "asd".into()), // invalid case
        ("../non/*".into(), "asd".into()),             // invalid case
      ]),
      true,
    )
    .iter()
    .flatten()
    .collect::<Vec<_>>();

    let expected = expected_resources(&[
      ("../src/script.js", "main.js"),
      ("../src/assets/javascript.svg", "javascript.svg"),
      ("../src/assets/tauri.svg", "tauri.svg"),
      ("../src/assets/rust.svg", "rust.svg"),
      ("../src/assets/lang/en.json", "lang/en.json"),
      ("../src/assets/lang/ar.json", "lang/ar.json"),
      ("../src/index.html", "frontend/index.html"),
      ("../src/sounds/lang/es.wav", "voices/lang/es.wav"),
      ("../src/sounds/lang/fr.wav", "voices/lang/fr.wav"),
      ("../src/textures/water.tex", "textures/water.tex"),
      ("../src/textures/fire.tex", "textures/fire.tex"),
      ("../src/tiles/grass.tile", "tiles/grass.tile"),
      ("../src/tiles/stones.tile", "tiles/stones.tile"),
      ("../src/tiles/sky/grey.tile", "tiles/grey.tile"),
      ("../src/tiles/sky/yellow.tile", "tiles/yellow.tile"),
      ("Cargo.toml", "Cargo.toml"),
      ("Tauri.toml", "Tauri.toml"),
      ("tauri.conf.json", "json/tauri.conf.json"),
    ]);

    assert_eq!(resources.len(), expected.len());
    for resource in expected {
      if !resources.contains(&resource) {
        panic!("{resource:?} was expected but not found in {resources:?}");
      }
    }
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_map_no_walk() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::from_map(
      &std::collections::HashMap::from_iter([
        ("../src/script.js".into(), "main.js".into()),
        ("../src/assets".into(), "".into()),
        ("../src/index.html".into(), "frontend/index.html".into()),
        ("../src/sounds".into(), "voices".into()),
        ("*.toml".into(), "".into()),
        ("*.conf.json".into(), "json".into()),
      ]),
      false,
    )
    .iter()
    .flatten()
    .collect::<Vec<_>>();

    let expected = expected_resources(&[
      ("../src/script.js", "main.js"),
      ("../src/index.html", "frontend/index.html"),
      ("Cargo.toml", "Cargo.toml"),
      ("Tauri.toml", "Tauri.toml"),
      ("tauri.conf.json", "json/tauri.conf.json"),
    ]);

    assert_eq!(resources.len(), expected.len());
    for resource in expected {
      if !resources.contains(&resource) {
        panic!("{resource:?} was expected but not found in {resources:?}");
      }
    }
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_errors() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::from_map(
      &std::collections::HashMap::from_iter([
        ("../non-existent-file".into(), "file".into()),
        ("../non-existent-dir".into(), "dir".into()),
        // exists but not allowed to walk
        ("../src".into(), "dir2".into()),
        // doesn't exist but it is a glob and will return an error
        ("../non-existent-glob-dir/*".into(), "glob".into()),
        // exists but only contains directories and will not produce any values
        ("../src/dir/*".into(), "dir3".into()),
      ]),
      false,
    )
    .iter()
    .collect::<Vec<_>>();

    assert_eq!(resources.len(), 4);

    assert!(resources.iter().all(|r| r.is_err()));

    // hashmap order is not guaranteed so we check the error variant exists and how many
    assert!(resources
      .iter()
      .any(|r| matches!(r, Err(crate::Error::ResourcePathNotFound(_)))));
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::ResourcePathNotFound(_))))
        .count(),
      2
    );
    assert!(resources
      .iter()
      .any(|r| matches!(r, Err(crate::Error::NotAllowedToWalkDir(_)))));
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::NotAllowedToWalkDir(_))))
        .count(),
      1
    );
    assert!(resources
      .iter()
      .any(|r| matches!(r, Err(crate::Error::GlobPathNotFound(_)))));
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::GlobPathNotFound(_))))
        .count(),
      1
    );
  }
}
