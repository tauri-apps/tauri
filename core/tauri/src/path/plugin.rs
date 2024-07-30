// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

use serialize_to_javascript::{default_template, DefaultTemplate, Template};

use super::{BaseDirectory, Error, PathResolver, Result};
use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, Runtime, State,
};

/// Normalize a path, removing things like `.` and `..`, this snippet is taken from cargo's paths util.
/// <https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106>
fn normalize_path(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    components.next();
    PathBuf::from(c.as_os_str())
  } else {
    PathBuf::new()
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        ret.push(c);
      }
    }
  }
  ret
}

/// Normalize a path, removing things like `.` and `..`, this snippet is taken from cargo's paths util but
/// slightly modified to not resolve absolute paths.
/// <https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106>
fn normalize_path_no_absolute(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    components.next();
    PathBuf::from(c.as_os_str())
  } else {
    PathBuf::new()
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        // Using PathBuf::push here will replace the whole path if an absolute path is encountered
        // which is not the intended behavior, so instead of that, convert the current resolved path
        // to a string and do simple string concatenation with the current component then convert it
        // back to a PathBuf
        let mut p = ret.to_string_lossy().to_string();
        // Only add a separator if it doesn't have one already or if current normalized path is empty,
        // this ensures it won't have an unwanted leading separator
        if !p.is_empty() && !p.ends_with('/') && !p.ends_with('\\') {
          p.push(MAIN_SEPARATOR);
        }
        if let Some(c) = c.to_str() {
          p.push_str(c);
        }
        ret = PathBuf::from(p);
      }
    }
  }
  ret
}

#[command(root = "crate")]
pub fn resolve_directory<R: Runtime>(
  _app: AppHandle<R>,
  resolver: State<'_, PathResolver<R>>,
  directory: BaseDirectory,
  path: Option<PathBuf>,
) -> Result<PathBuf> {
  super::resolve_path(&resolver, directory, path).map(|p| dunce::simplified(&p).to_path_buf())
}

#[command(root = "crate")]
pub fn resolve(paths: Vec<String>) -> Result<PathBuf> {
  // Start with current directory then start adding paths from the vector one by one using `PathBuf.push()` which
  // will ensure that if an absolute path is encountered in the iteration, it will be used as the current full path.
  //
  // examples:
  // 1. `vec!["."]` or `vec![]` will be equal to `std::env::current_dir()`
  // 2. `vec!["/foo/bar", "/tmp/file", "baz"]` will be equal to `PathBuf::from("/tmp/file/baz")`
  let mut path = std::env::current_dir().map_err(Error::CurrentDir)?;
  for p in paths {
    path.push(p);
  }
  Ok(dunce::simplified(&normalize_path(&path)).to_path_buf())
}

#[command(root = "crate")]
pub fn normalize(path: String) -> String {
  let mut p = dunce::simplified(&normalize_path_no_absolute(Path::new(&path)))
    .to_string_lossy()
    .to_string();

  // Node.js behavior is to return `".."` for `normalize("..")`
  // and `"."` for `normalize("")` or `normalize(".")`
  if p.is_empty() && path == ".." {
    "..".into()
  } else if p.is_empty() && path == "." {
    ".".into()
  } else {
    // Add a trailing separator if the path passed to this functions had a trailing separator. That's how Node.js behaves.
    if (path.ends_with('/') || path.ends_with('\\')) && (!p.ends_with('/') || !p.ends_with('\\')) {
      p.push(MAIN_SEPARATOR);
    }
    p
  }
}

#[command(root = "crate")]
pub fn join(mut paths: Vec<String>) -> String {
  let path = PathBuf::from(
    paths
      .iter_mut()
      .map(|p| {
        // Add a `MAIN_SEPARATOR` if it doesn't already have one.
        // Doing this to ensure that the vector elements are separated in
        // the resulting string so path.components() can work correctly when called
        // in `normalize_path_no_absolute()` later on.
        if !p.ends_with('/') && !p.ends_with('\\') {
          p.push(MAIN_SEPARATOR);
        }
        p.to_string()
      })
      .collect::<String>(),
  );

  let p = dunce::simplified(&normalize_path_no_absolute(&path))
    .to_string_lossy()
    .to_string();

  if p.is_empty() {
    ".".into()
  } else {
    p
  }
}

#[command(root = "crate")]
pub fn dirname(path: String) -> Result<PathBuf> {
  match Path::new(&path).parent() {
    Some(p) => Ok(dunce::simplified(p).to_path_buf()),
    None => Err(Error::NoParent),
  }
}

#[command(root = "crate")]
pub fn extname(path: String) -> Result<String> {
  match Path::new(&path)
    .extension()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(p) => Ok(p.to_string()),
    None => Err(Error::NoExtension),
  }
}

#[command(root = "crate")]
pub fn basename(path: &str, ext: Option<&str>) -> Result<String> {
  let file_name = Path::new(path).file_name().map(|f| f.to_string_lossy());
  match file_name {
    Some(p) => {
      let maybe_stripped = if let Some(ext) = ext {
        p.strip_suffix(ext).unwrap_or(&p).to_string()
      } else {
        p.to_string()
      };
      Ok(maybe_stripped)
    }
    None => Err(Error::NoBasename),
  }
}

#[command(root = "crate")]
pub fn is_absolute(path: String) -> bool {
  Path::new(&path).is_absolute()
}

#[derive(Template)]
#[default_template("./init.js")]
struct InitJavascript {
  sep: &'static str,
  delimiter: &'static str,
}

/// Initializes the plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  #[cfg(windows)]
  let (sep, delimiter) = ("\\", ";");
  #[cfg(not(windows))]
  let (sep, delimiter) = ("/", ":");

  let init_js = InitJavascript { sep, delimiter }
    .render_default(&Default::default())
    // this will never fail with the above sep and delimiter values
    .unwrap();

  Builder::new("path")
    .invoke_handler(crate::generate_handler![
      resolve_directory,
      resolve,
      normalize,
      join,
      dirname,
      extname,
      basename,
      is_absolute
    ])
    .js_init_script(init_js.to_string())
    .setup(|app, _api| {
      #[cfg(target_os = "android")]
      {
        let handle = _api.register_android_plugin("app.tauri", "PathPlugin")?;
        app.manage(PathResolver(handle));
      }

      #[cfg(not(target_os = "android"))]
      {
        app.manage(PathResolver(app.clone()));
      }

      Ok(())
    })
    .build()
}

#[cfg(test)]
mod tests {

  #[test]
  fn basename() {
    let path = "/path/to/some-json-file.json";
    assert_eq!(
      super::basename(path, Some(".json")).unwrap(),
      "some-json-file"
    );

    let path = "/path/to/some-json-file.json";
    assert_eq!(
      super::basename(path, Some("json")).unwrap(),
      "some-json-file."
    );

    let path = "/path/to/some-json-file.html.json";
    assert_eq!(
      super::basename(path, Some(".json")).unwrap(),
      "some-json-file.html"
    );

    let path = "/path/to/some-json-file.json.json";
    assert_eq!(
      super::basename(path, Some(".json")).unwrap(),
      "some-json-file.json"
    );

    let path = "/path/to/some-json-file.json.html";
    assert_eq!(
      super::basename(path, Some(".json")).unwrap(),
      "some-json-file.json.html"
    );
  }
}
