// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  path::{Component, Path, PathBuf},
};

use walkdir::WalkDir;

use crate::platform::Target as TargetPlatform;

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

/// Normalizes a path to the resource *source* location, dropping the no-op
/// `.` components while keeping the path pointing at the same file.
///
/// Unlike [`resource_relpath`], the prefix and root are preserved: this is the
/// path the resource is read from, so dropping the `C:` of `C:\assets` would
/// leave a root-relative path that resolves against whatever drive the current
/// directory happens to be on.
fn normalize(path: &Path) -> PathBuf {
  let mut dest = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Prefix(_) | Component::RootDir => dest.push(component.as_os_str()),
      Component::CurDir => {}
      Component::ParentDir => dest.push(".."),
      Component::Normal(string) => dest.push(string),
    }
  }
  dest
}

/// Parses the external binaries to bundle, adding the target triple suffix to each of them.
pub fn external_binaries(
  external_binaries: &[String],
  target_triple: &str,
  target_platform: &TargetPlatform,
) -> Vec<String> {
  let mut paths = Vec::new();
  for curr_path in external_binaries {
    let extension = if matches!(target_platform, TargetPlatform::Windows) {
      ".exe"
    } else {
      ""
    };
    paths.push(format!("{curr_path}-{target_triple}{extension}"));
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
        current_dest: None,
        current_iter: None,
        rerun_if_changed: Vec::new(),
      },
    }
  }

  /// Creates a new ResourcePaths from a slice of patterns to iterate
  pub fn from_map(patterns: &'a HashMap<String, String>, allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      iter: ResourcePathsIter {
        pattern_iter: PatternIter::Map(patterns.iter()),
        allow_walk,
        current_dest: None,
        current_iter: None,
        rerun_if_changed: Vec::new(),
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

  /// The value of map when [`Self::pattern_iter`] is a [`PatternIter::Map`],
  /// used for determining [`Resource::target`]
  current_dest: Option<PathBuf>,
  /// The iter for the current pattern. The cycle goes like this:
  /// [`ResourcePaths::next`] -> [`Self::next`] -> [`Self::pattern_iter::next`] -> [`Self::current_iter::next`]
  current_iter: Option<ResourcePathsInnerIter>,
  /// Paths that were walked or globbed while iterating. Build scripts
  /// should emit a `rerun-if-changed` for each so that adding or removing a
  /// file inside a resource directory re-triggers the resource copy.
  rerun_if_changed: Vec<PathBuf>,
}

#[derive(Debug)]
enum ResourcePathsInnerIter {
  Walk {
    iter: walkdir::IntoIter,
    /// The key of map when [`ResourcePathsIter::pattern_iter`] is a [`PatternIter::Map`],
    /// used for determining [`Resource::target`]
    current_pattern: Option<PathBuf>,
  },
  Glob {
    iter: glob::Paths,
  },
}

impl Iterator for ResourcePathsInnerIter {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    match self {
      ResourcePathsInnerIter::Walk { iter, .. } => Some(
        iter
          .next()?
          .map(|entry| entry.into_path())
          .map_err(Into::into),
      ),
      ResourcePathsInnerIter::Glob { iter } => Some(iter.next()?.map_err(Into::into)),
    }
  }
}

impl ResourcePathsIter<'_> {
  /// Paths that were walked or globbed while iterating.
  ///
  /// A build script should emit a `cargo:rerun-if-changed` for each of these
  /// after iterating, so that adding or removing a file inside a resource
  /// directory re-runs the script and copies the new files. Only meaningful
  /// once iteration has produced the entries (i.e. after the iterator is
  /// exhausted).
  pub fn rerun_if_changed(&self) -> &[PathBuf] {
    &self.rerun_if_changed
  }

  fn next_current_iter(&mut self) -> Option<crate::Result<Resource>> {
    let current_iter = self.current_iter.as_mut().unwrap();
    let entry = current_iter.next()?;

    Some(match entry {
      Ok(entry) => {
        // Skip directories
        if entry.is_dir() {
          self.next_current_iter()?
        } else {
          self.resource_from_path(normalize(&entry))
        }
      }
      Err(error) => Err(error),
    })
  }

  fn resource_from_path(&self, path: PathBuf) -> crate::Result<Resource> {
    if !path.exists() {
      return Err(crate::Error::ResourcePathNotFound(path));
    }

    Ok(Resource {
      target: if let Some(dest) = &self.current_dest {
        match &self.current_iter {
          Some(current_iter) => match current_iter {
            // if processing a directory, preserve directory structure under current_dest
            ResourcePathsInnerIter::Walk {
              current_pattern, ..
            } => {
              // `current_pattern` is always set when `current_dest` is, since both
              // only exist for the map form, and walked entries always start with
              // the pattern they were walked from, so this is the entry path
              // relative to the pattern.
              //
              // `resource_relpath` is the fallback rather than `path` itself
              // because it is guaranteed to be relative: joining an absolute
              // path would discard `dest` entirely.
              let relpath = current_pattern
                .as_deref()
                .and_then(|pattern| path.strip_prefix(pattern).ok())
                .map(PathBuf::from)
                .unwrap_or_else(|| resource_relpath(&path));
              dest.join(relpath)
            }
            // if processing a glob and current_dest is not empty
            // we put all globbed paths under current_dest
            // preserving the file name as it is
            ResourcePathsInnerIter::Glob { .. } => dest.join(path.file_name().unwrap()),
          },
          None => {
            if dest.components().count() == 0 {
              // if current_dest is empty while processing a file pattern
              // we preserve the file name as it is
              //
              // e.g. `{ "README.md": "" }` is `README.md` -> `$RESOURCE/README.md`
              //
              // TODO: This behavior is a confusing special case,
              // remove this in v3 or make other cases like this work
              // > `{ "README.md": "./folder/" }` is `README.md` -> `$RESOURCE/folder/README.md` (this gives `$RESOURCE/folder` today)
              PathBuf::from(path.file_name().unwrap())
            } else {
              dest.clone()
            }
          }
        }
      } else {
        // If [`ResourcePathsIter::pattern_iter`] is a [`PatternIter::Slice`]
        resource_relpath(&path)
      },
      path,
    })
  }

  fn next_pattern(&mut self) -> Option<crate::Result<Resource>> {
    self.current_dest = None;
    self.current_iter = None;

    let pattern = match &mut self.pattern_iter {
      PatternIter::Slice(iter) => iter.next()?,
      PatternIter::Map(iter) => {
        let (pattern, dest) = iter.next()?;
        self.current_dest = Some(resource_relpath(Path::new(dest)));
        pattern
      }
    };

    if pattern.contains('*') {
      // Watch the fixed directory prefix of the glob (everything before the
      // first wildcard component) so new files matching the glob are noticed.
      let mut base = PathBuf::new();
      for component in Path::new(pattern).components() {
        if component.as_os_str().to_string_lossy().contains('*') {
          break;
        }
        base.push(component);
      }
      if base.as_os_str().is_empty() {
        base.push(".");
      }
      self.rerun_if_changed.push(base);

      self.current_iter = match glob::glob(pattern) {
        Ok(glob) => Some(ResourcePathsInnerIter::Glob { iter: glob }),
        Err(error) => return Some(Err(error.into())),
      };
      match self.next_current_iter() {
        Some(r) => Some(r),
        None => {
          self.current_iter = None;
          Some(Err(crate::Error::GlobPathNotFound(pattern.clone())))
        }
      }
    } else {
      let path = normalize(Path::new(pattern));
      self.rerun_if_changed.push(path.clone());
      if path.is_dir() {
        if !self.allow_walk {
          return Some(Err(crate::Error::NotAllowedToWalkDir(path)));
        }
        self.current_iter = Some(ResourcePathsInnerIter::Walk {
          iter: WalkDir::new(&path).into_iter(),
          current_pattern: if matches!(self.pattern_iter, PatternIter::Map(_)) {
            Some(path)
          } else {
            None
          },
        });
        // If the directory is empty, skip and continue to the next pattern
        self.next_current_iter().or_else(|| self.next_pattern())
      } else {
        Some(self.resource_from_path(path))
      }
    }
  }
}

impl Iterator for ResourcePaths<'_> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    self.iter.next().map(|r| r.map(|res| res.path))
  }
}

impl Iterator for ResourcePathsIter<'_> {
  type Item = crate::Result<Resource>;

  fn next(&mut self) -> Option<crate::Result<Resource>> {
    if self.current_iter.is_some() {
      match self.next_current_iter() {
        Some(r) => return Some(r),
        None => self.current_iter = None,
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
    getrandom::fill(&mut random).unwrap();

    let temp = std::env::temp_dir();
    let temp = temp.join(format!("tauri_resource_paths_iter_test_{}", random[0]));

    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();

    std::env::set_current_dir(&temp).unwrap();

    let paths = [
      "src-tauri/tauri.conf.json",
      "src-tauri/some-other-json.json",
      "src-tauri/Cargo.toml",
      "src-tauri/Tauri.toml",
      "src-tauri/build.rs",
      "src-tauri/some-folder/some-file.txt",
      "src/assets/javascript.svg",
      "src/assets/tauri.svg",
      "src/assets/rust.svg",
      "src/assets/lang/en.json",
      "src/assets/lang/ar.json",
      "src/sounds/lang/es.wav",
      "src/sounds/lang/fr.wav",
      "src/textures/ground/earth.tex",
      "src/textures/ground/sand.tex",
      "src/textures/water.tex",
      "src/textures/fire.tex",
      "src/tiles/sky/grey.tile",
      "src/tiles/sky/yellow.tile",
      "src/tiles/grass.tile",
      "src/tiles/stones.tile",
      "src/index.html",
      "src/style.css",
      "src/script.js",
      "src/dir/another-dir/file1.txt",
      "src/dir/another-dir2/file2.txt",
    ];

    for path in paths {
      let path = Path::new(path);
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, "").unwrap();
    }
    fs::create_dir_all("empty-directory").unwrap();
  }

  fn resources_map(literal: &[(&str, &str)]) -> HashMap<String, String> {
    literal
      .iter()
      .map(|(from, to)| (from.to_string(), to.to_string()))
      .collect()
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_slice_allow_walk() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let resources = ResourcePaths::new(
      &[
        // `empty-directory` should not affect anything
        "../empty-directory".into(),
        "../src/script.js".into(),
        "../src/assets".into(),
        "../src/index.html".into(),
        "../src/sounds".into(),
        // Should be the same as `../src/textures/` or `../src/textures`
        "../src/textures/**/*".into(),
        "*.toml".into(),
        "*.conf.json".into(),
      ],
      true,
    )
    .iter()
    .flatten()
    .collect::<Vec<_>>();

    let expected = expected_resources(&[
      // From `../src/script.js`
      ("../src/script.js", "_up_/src/script.js"),
      // From `../src/assets`
      (
        "../src/assets/javascript.svg",
        "_up_/src/assets/javascript.svg",
      ),
      ("../src/assets/tauri.svg", "_up_/src/assets/tauri.svg"),
      ("../src/assets/rust.svg", "_up_/src/assets/rust.svg"),
      ("../src/assets/lang/en.json", "_up_/src/assets/lang/en.json"),
      ("../src/assets/lang/ar.json", "_up_/src/assets/lang/ar.json"),
      // From `../src/index.html`
      ("../src/index.html", "_up_/src/index.html"),
      // From `../src/sounds`
      ("../src/sounds/lang/es.wav", "_up_/src/sounds/lang/es.wav"),
      ("../src/sounds/lang/fr.wav", "_up_/src/sounds/lang/fr.wav"),
      // From `../src/textures/**/*`
      (
        "../src/textures/ground/earth.tex",
        "_up_/src/textures/ground/earth.tex",
      ),
      (
        "../src/textures/ground/sand.tex",
        "_up_/src/textures/ground/sand.tex",
      ),
      ("../src/textures/water.tex", "_up_/src/textures/water.tex"),
      ("../src/textures/fire.tex", "_up_/src/textures/fire.tex"),
      // From `*.toml`
      ("Cargo.toml", "Cargo.toml"),
      ("Tauri.toml", "Tauri.toml"),
      // From `*.conf.json`
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
  fn resource_paths_iter_rerun_if_changed() {
    setup_test_dirs();

    let dir = std::env::current_dir().unwrap().join("src-tauri");
    let _ = std::env::set_current_dir(dir);

    let patterns = [
      "../src/script.js".into(),
      "../src/assets".into(),
      "../src/textures/**/*".into(),
      "*.toml".into(),
    ];
    let mut resources = ResourcePaths::new(&patterns, true).iter();

    for resource in resources.by_ref() {
      resource.unwrap();
    }

    assert_eq!(
      resources.rerun_if_changed(),
      &[
        PathBuf::from("../src/script.js"),
        PathBuf::from("../src/assets"),
        PathBuf::from("../src/textures"),
        PathBuf::from("."),
      ]
    );
  }

  #[test]
  #[serial_test::serial(resources)]
  fn resource_paths_iter_absolute_paths() {
    setup_test_dirs();

    // `setup_test_dirs` moves into the temp directory, so these are absolute
    // paths, which on Windows means they carry a drive letter prefix.
    let root = std::env::current_dir().unwrap();
    let file = root.join("src").join("index.html");
    let dir = root.join("src").join("sounds");

    let patterns = [
      file.to_string_lossy().into_owned(),
      dir.to_string_lossy().into_owned(),
    ];
    let mut iter = ResourcePaths::new(&patterns, true).iter();
    let resources = iter.by_ref().flatten().collect::<Vec<_>>();

    let mut paths = resources
      .iter()
      .map(|resource| resource.path().to_path_buf())
      .collect::<Vec<_>>();
    paths.sort();

    let mut expected = vec![
      file.clone(),
      dir.join("lang").join("es.wav"),
      dir.join("lang").join("fr.wav"),
    ];
    expected.sort();

    assert_eq!(paths, expected);

    for resource in &resources {
      // On Windows a path is only absolute when it has a prefix *and* a root,
      // so this is what catches a dropped drive letter: `\src\index.html`
      // would resolve against whatever drive the current directory is on.
      assert!(
        resource.path().is_absolute(),
        "{:?} is not absolute, it would resolve against the current drive",
        resource.path()
      );
      // The in-bundle target is unaffected: the root still maps to `_root_`,
      // and it must stay relative for the bundlers to join it onto the
      // resources directory.
      assert!(
        resource.target().is_relative() && resource.target().starts_with("_root_"),
        "unexpected target {:?} for {:?}",
        resource.target(),
        resource.path()
      );
    }

    assert_eq!(iter.rerun_if_changed(), &[file.clone(), dir.clone()]);

    // the map form must still place the resources under the given destination
    let dir_pattern = dir.to_string_lossy().into_owned();
    let resources = ResourcePaths::from_map(&resources_map(&[(&dir_pattern, "voices")]), true)
      .iter()
      .flatten()
      .collect::<Vec<_>>();

    let mut targets = resources
      .iter()
      .map(|resource| resource.target().to_path_buf())
      .collect::<Vec<_>>();
    targets.sort();

    assert_eq!(
      targets,
      [
        PathBuf::from("voices").join("lang").join("es.wav"),
        PathBuf::from("voices").join("lang").join("fr.wav"),
      ]
    );

    for resource in &resources {
      assert!(
        resource.path().is_absolute(),
        "{:?} is not absolute, it would resolve against the current drive",
        resource.path()
      );
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
      &resources_map(&[
        ("../src/script.js", "main.js"),
        ("../src/assets", ""),
        ("../src/index.html", "frontend/index.html"),
        ("../src/sounds", "voices"),
        ("../src/textures/*", "textures"),
        ("../src/tiles/**/*", "tiles"),
        ("*.toml", ""),
        ("*.conf.json", "json"),
        ("./some-folder/", "some-target-folder/"),
        ("../non-existent-file", "asd"), // invalid case
        ("../non/*", "asd"),             // invalid case
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
      (
        "some-folder/some-file.txt",
        "some-target-folder/some-file.txt",
      ),
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
      &resources_map(&[
        ("../src/script.js", "main.js"),
        ("../src/assets", ""),
        ("../src/index.html", "frontend/index.html"),
        ("../src/sounds", "voices"),
        ("*.toml", ""),
        ("*.conf.json", "json"),
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
      &resources_map(&[
        ("../non-existent-file", "file"),
        ("../non-existent-dir", "dir"),
        // exists but not allowed to walk
        ("../src", "dir2"),
        // doesn't exist but it is a glob and will return an error
        ("../non-existent-glob-dir/*", "glob"),
        // exists but only contains directories and will not produce any values
        ("../src/dir/*", "dir3"),
      ]),
      false,
    )
    .iter()
    .collect::<Vec<_>>();

    assert_eq!(resources.len(), 5);

    assert!(resources.iter().all(|r| r.is_err()));

    // hashmap order is not guaranteed so we check the error variant exists and how many
    assert!(
      resources
        .iter()
        .any(|r| matches!(r, Err(crate::Error::ResourcePathNotFound(_))))
    );
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::ResourcePathNotFound(_))))
        .count(),
      2
    );
    assert!(
      resources
        .iter()
        .any(|r| matches!(r, Err(crate::Error::NotAllowedToWalkDir(_))))
    );
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::NotAllowedToWalkDir(_))))
        .count(),
      1
    );
    assert!(
      resources
        .iter()
        .any(|r| matches!(r, Err(crate::Error::GlobPathNotFound(_))))
    );
    assert_eq!(
      resources
        .iter()
        .filter(|r| matches!(r, Err(crate::Error::GlobPathNotFound(_))))
        .count(),
      2
    );
  }
}
