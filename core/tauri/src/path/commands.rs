use std::{
  env::temp_dir,
  path::{Component, Path, PathBuf, MAIN_SEPARATOR},
};

use super::{BaseDirectory, Error, PathResolver, Result};
use crate::{AppHandle, Runtime, State};

/// Normalize a path, removing things like `.` and `..`, this snippet is taken from cargo's paths util.
/// https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106
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
/// https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106
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

pub(crate) fn resolve_path<R: Runtime>(
  resolver: State<'_, PathResolver<R>>,
  directory: BaseDirectory,
  path: Option<PathBuf>,
) -> Result<PathBuf> {
  let resolve_resource = matches!(directory, BaseDirectory::Resource);
  let mut base_dir_path = match directory {
    BaseDirectory::Audio => resolver.audio_dir(),
    BaseDirectory::Cache => resolver.cache_dir(),
    BaseDirectory::Config => resolver.config_dir(),
    BaseDirectory::Data => resolver.data_dir(),
    BaseDirectory::LocalData => resolver.local_data_dir(),
    BaseDirectory::Document => resolver.document_dir(),
    BaseDirectory::Download => resolver.download_dir(),
    BaseDirectory::Picture => resolver.picture_dir(),
    BaseDirectory::Public => resolver.public_dir(),
    BaseDirectory::Video => resolver.video_dir(),
    BaseDirectory::Resource => resolver.resource_dir(),
    BaseDirectory::Temp => Ok(temp_dir()),
    BaseDirectory::AppConfig => resolver.app_config_dir(),
    BaseDirectory::AppData => resolver.app_data_dir(),
    BaseDirectory::AppLocalData => resolver.app_local_data_dir(),
    BaseDirectory::AppCache => resolver.app_cache_dir(),
    BaseDirectory::AppLog => resolver.app_log_dir(),
    #[cfg(desktop)]
    BaseDirectory::Desktop => resolver.desktop_dir(),
    #[cfg(desktop)]
    BaseDirectory::Executable => resolver.executable_dir(),
    #[cfg(desktop)]
    BaseDirectory::Font => resolver.font_dir(),
    #[cfg(desktop)]
    BaseDirectory::Home => resolver.home_dir(),
    #[cfg(desktop)]
    BaseDirectory::Runtime => resolver.runtime_dir(),
    #[cfg(desktop)]
    BaseDirectory::Template => resolver.template_dir(),
  }?;

  if let Some(path) = path {
    // use the same path resolution mechanism as the bundler's resource injection algorithm
    if resolve_resource {
      let mut resource_path = PathBuf::new();
      for component in path.components() {
        match component {
          Component::Prefix(_) => {}
          Component::RootDir => resource_path.push("_root_"),
          Component::CurDir => {}
          Component::ParentDir => resource_path.push("_up_"),
          Component::Normal(p) => resource_path.push(p),
        }
      }
      base_dir_path.push(resource_path);
    } else {
      base_dir_path.push(path);
    }
  }

  Ok(base_dir_path)
}

#[crate::command(root = "crate")]
pub fn resolve_directory<R: Runtime>(
  _app: AppHandle<R>,
  resolver: State<'_, PathResolver<R>>,
  directory: BaseDirectory,
  path: Option<PathBuf>,
) -> Result<PathBuf> {
  resolve_path(resolver, directory, path)
}

#[crate::command(root = "crate")]
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
  Ok(normalize_path(&path))
}

#[crate::command(root = "crate")]
pub fn normalize(path: String) -> String {
  let mut p = normalize_path_no_absolute(Path::new(&path))
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

#[crate::command(root = "crate")]
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

  let p = normalize_path_no_absolute(&path)
    .to_string_lossy()
    .to_string();
  if p.is_empty() {
    ".".into()
  } else {
    p
  }
}

#[crate::command(root = "crate")]
pub fn dirname(path: String) -> Result<PathBuf> {
  match Path::new(&path).parent() {
    Some(p) => Ok(p.to_path_buf()),
    None => Err(Error::NoParent),
  }
}

#[crate::command(root = "crate")]
pub fn extname(path: String) -> Result<String> {
  match Path::new(&path)
    .extension()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(p) => Ok(p.to_string()),
    None => Err(Error::NoExtension),
  }
}

#[crate::command(root = "crate")]
pub fn basename(path: String, ext: Option<String>) -> Result<String> {
  match Path::new(&path)
    .file_name()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(p) => Ok(if let Some(ext) = ext {
      p.replace(ext.as_str(), "")
    } else {
      p.to_string()
    }),
    None => Err(Error::NoBasename),
  }
}

#[crate::command(root = "crate")]
pub fn is_absolute(path: String) -> bool {
  Path::new(&path).is_absolute()
}
