// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{HashMap, HashSet},
  fmt,
  path::{Path, PathBuf, MAIN_SEPARATOR},
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
  },
};

use tauri_utils::config::FsScope;

use crate::ScopeEventId;

pub use glob::Pattern;

/// Scope change event.
#[derive(Debug, Clone)]
pub enum Event {
  /// A path has been allowed.
  PathAllowed(PathBuf),
  /// A path has been forbidden.
  PathForbidden(PathBuf),
}

type EventListener = Box<dyn Fn(&Event) + Send>;

/// Scope for filesystem access.
#[derive(Clone)]
pub struct Scope {
  allowed_patterns: Arc<Mutex<HashSet<Pattern>>>,
  forbidden_patterns: Arc<Mutex<HashSet<Pattern>>>,
  event_listeners: Arc<Mutex<HashMap<ScopeEventId, EventListener>>>,
  match_options: glob::MatchOptions,
  next_event_id: Arc<AtomicU32>,
}

impl Scope {
  fn next_event_id(&self) -> u32 {
    self.next_event_id.fetch_add(1, Ordering::Relaxed)
  }
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Scope")
      .field(
        "allowed_patterns",
        &self
          .allowed_patterns
          .lock()
          .unwrap()
          .iter()
          .map(|p| p.as_str())
          .collect::<Vec<&str>>(),
      )
      .field(
        "forbidden_patterns",
        &self
          .forbidden_patterns
          .lock()
          .unwrap()
          .iter()
          .map(|p| p.as_str())
          .collect::<Vec<&str>>(),
      )
      .finish()
  }
}

fn push_pattern<P: AsRef<Path>, F: Fn(&str) -> Result<Pattern, glob::PatternError>>(
  list: &mut HashSet<Pattern>,
  pattern: P,
  f: F,
) -> crate::Result<()> {
  // Reconstruct pattern path components with appropraite separator
  // so `some\path/to/dir/**\*` would be `some/path/to/dir/**/*` on Unix
  // and  `some\path\to\dir\**\*` on Windows.
  let path: PathBuf = pattern.as_ref().components().collect();

  // Add pattern as is to be matched with paths as is
  let path_str = path.to_string_lossy();
  list.insert(f(&path_str)?);

  // On Windows, if path starts with a Prefix, try to strip it if possible
  // so `\\?\C:\\SomeDir` would result in a scope of:
  //   - `\\?\C:\\SomeDir`
  //   - `C:\\SomeDir`
  #[cfg(windows)]
  {
    use std::path::{Component, Path, Prefix};

    let mut components = path.components();

    let is_unc = match components.next() {
      Some(Component::Prefix(p)) => match p.kind() {
        Prefix::VerbatimDisk(..) => true,
        _ => false, // Other kinds of UNC paths
      },
      _ => false, // relative or empty
    };

    if is_unc {
      // we remove UNC manually, instead of `dunce::simplified` because
      // `path` could have `*` in it and that's not allowed on Windows and
      // `dunce::simplified` will check that and return `path` as is without simplification
      let simplified = path
        .to_str()
        .and_then(|s| s.get(4..))
        .map_or(path.as_path(), Path::new);

      let simplified_str = simplified.to_string_lossy();
      if simplified_str != path_str {
        list.insert(f(&simplified_str)?);
      }
    }
  }

  // Add canonicalized version of the pattern or canonicalized version of its parents
  // so `/data/user/0/appid/assets/*` would be canonicalized to `/data/data/appid/assets/*`
  // and can then be matched against any of them.
  if let Some(p) = canonicalize_parent(path) {
    list.insert(f(&p.to_string_lossy())?);
  }

  Ok(())
}

/// Attempt to canonicalize path or its parents in case we have a path like `/data/user/0/appid/**`
/// where `**` obviously does not exist but we need to canonicalize the parent.
///
/// example: given the `/data/user/0/appid/assets/*` path,
/// it's a glob pattern so it won't exist (std::fs::canonicalize() fails);
///
/// the second iteration needs to check `/data/user/0/appid/assets` and save the `*` component to append later.
///
/// if it also does not exist, a third iteration is required to check `/data/user/0/appid`
/// with `assets/*` as the cached value (`checked_path` variable)
/// on Android that gets canonicalized to `/data/data/appid` so the final value will be `/data/data/appid/assets/*`
/// which is the value we want to check when we execute the `Scope::is_allowed` function
fn canonicalize_parent(mut path: PathBuf) -> Option<PathBuf> {
  let mut failed_components = None;

  loop {
    if let Ok(path) = path.canonicalize() {
      break Some(if let Some(p) = failed_components {
        path.join(p)
      } else {
        path
      });
    }

    // grap the last component of the path
    if let Some(mut last) = path.iter().next_back().map(PathBuf::from) {
      // remove the last component of the path so the next iteration checks its parent
      // if there is no more parent components, we failed to canonicalize
      if !path.pop() {
        break None;
      }

      // append the already checked path to the last component
      // to construct `<last>/<checked_path>` and saved it for next iteration
      if let Some(failed_components) = &failed_components {
        last.push(failed_components);
      }
      failed_components.replace(last);
    } else {
      break None;
    }
  }
}
impl Scope {
  /// Creates a new scope from a [`FsScope`] configuration.
  pub fn new<R: crate::Runtime, M: crate::Manager<R>>(
    manager: &M,
    scope: &FsScope,
  ) -> crate::Result<Self> {
    let mut allowed_patterns = HashSet::new();
    for path in scope.allowed_paths() {
      if let Ok(path) = manager.path().parse(path) {
        push_pattern(&mut allowed_patterns, path, Pattern::new)?;
      }
    }

    let mut forbidden_patterns = HashSet::new();
    if let Some(forbidden_paths) = scope.forbidden_paths() {
      for path in forbidden_paths {
        if let Ok(path) = manager.path().parse(path) {
          push_pattern(&mut forbidden_patterns, path, Pattern::new)?;
        }
      }
    }

    let require_literal_leading_dot = match scope {
      FsScope::Scope {
        require_literal_leading_dot: Some(require),
        ..
      } => *require,
      // dotfiles are not supposed to be exposed by default on unix
      #[cfg(unix)]
      _ => true,
      #[cfg(windows)]
      _ => false,
    };

    Ok(Self {
      allowed_patterns: Arc::new(Mutex::new(allowed_patterns)),
      forbidden_patterns: Arc::new(Mutex::new(forbidden_patterns)),
      event_listeners: Default::default(),
      next_event_id: Default::default(),
      match_options: glob::MatchOptions {
        // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
        // see: <https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5>
        require_literal_separator: true,
        require_literal_leading_dot,
        ..Default::default()
      },
    })
  }

  /// The list of allowed patterns.
  pub fn allowed_patterns(&self) -> HashSet<Pattern> {
    self.allowed_patterns.lock().unwrap().clone()
  }

  /// The list of forbidden patterns.
  pub fn forbidden_patterns(&self) -> HashSet<Pattern> {
    self.forbidden_patterns.lock().unwrap().clone()
  }

  /// Listen to an event on this scope.
  pub fn listen<F: Fn(&Event) + Send + 'static>(&self, f: F) -> ScopeEventId {
    let id = self.next_event_id();
    self.listen_with_id(id, f);
    id
  }

  fn listen_with_id<F: Fn(&Event) + Send + 'static>(&self, id: ScopeEventId, f: F) {
    self.event_listeners.lock().unwrap().insert(id, Box::new(f));
  }

  /// Listen to an event on this scope and immediately unlisten.
  pub fn once<F: FnOnce(&Event) + Send + 'static>(&self, f: F) -> ScopeEventId {
    let listerners = self.event_listeners.clone();
    let handler = std::cell::Cell::new(Some(f));
    let id = self.next_event_id();
    self.listen_with_id(id, move |event| {
      listerners.lock().unwrap().remove(&id);
      let handler = handler
        .take()
        .expect("attempted to call handler more than once");
      handler(event)
    });
    id
  }

  /// Removes an event listener on this scope.
  pub fn unlisten(&self, id: ScopeEventId) {
    self.event_listeners.lock().unwrap().remove(&id);
  }

  fn emit(&self, event: Event) {
    let listeners = self.event_listeners.lock().unwrap();
    let handlers = listeners.values();
    for listener in handlers {
      listener(&event);
    }
  }

  /// Extend the allowed patterns with the given directory.
  ///
  /// After this function has been called, the frontend will be able to use the Tauri API to read
  /// the directory and all of its files. If `recursive` is `true`, subdirectories will be accessible too.
  pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
    let path = path.as_ref();
    {
      let mut list = self.allowed_patterns.lock().unwrap();

      // allow the directory to be read
      push_pattern(&mut list, path, escaped_pattern)?;
      // allow its files and subdirectories to be read
      push_pattern(&mut list, path, |p| {
        escaped_pattern_with(p, if recursive { "**" } else { "*" })
      })?;
    }
    self.emit(Event::PathAllowed(path.to_path_buf()));
    Ok(())
  }

  /// Extend the allowed patterns with the given file path.
  ///
  /// After this function has been called, the frontend will be able to use the Tauri API to read the contents of this file.
  pub fn allow_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    let path = path.as_ref();
    push_pattern(
      &mut self.allowed_patterns.lock().unwrap(),
      path,
      escaped_pattern,
    )?;
    self.emit(Event::PathAllowed(path.to_path_buf()));
    Ok(())
  }

  /// Set the given directory path to be forbidden by this scope.
  ///
  /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
  pub fn forbid_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
    let path = path.as_ref();
    {
      let mut list = self.forbidden_patterns.lock().unwrap();

      // allow the directory to be read
      push_pattern(&mut list, path, escaped_pattern)?;
      // allow its files and subdirectories to be read
      push_pattern(&mut list, path, |p| {
        escaped_pattern_with(p, if recursive { "**" } else { "*" })
      })?;
    }
    self.emit(Event::PathForbidden(path.to_path_buf()));
    Ok(())
  }

  /// Set the given file path to be forbidden by this scope.
  ///
  /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
  pub fn forbid_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    let path = path.as_ref();
    push_pattern(
      &mut self.forbidden_patterns.lock().unwrap(),
      path,
      escaped_pattern,
    )?;
    self.emit(Event::PathForbidden(path.to_path_buf()));
    Ok(())
  }

  /// Determines if the given path is allowed on this scope.
  ///
  /// Returns `false` if the path was explicitly forbidden or neither allowed nor forbidden.
  ///
  /// May return `false` if the path points to a broken symlink.
  pub fn is_allowed<P: AsRef<Path>>(&self, path: P) -> bool {
    let path = try_resolve_symlink_and_canonicalize(path);

    if let Ok(path) = path {
      let path: PathBuf = path.components().collect();
      let forbidden = self
        .forbidden_patterns
        .lock()
        .unwrap()
        .iter()
        .any(|p| p.matches_path_with(&path, self.match_options));

      if forbidden {
        false
      } else {
        let allowed = self
          .allowed_patterns
          .lock()
          .unwrap()
          .iter()
          .any(|p| p.matches_path_with(&path, self.match_options));

        allowed
      }
    } else {
      false
    }
  }

  /// Determines if the given path is explicitly forbidden on this scope.
  ///
  /// May return `true` if the path points to a broken symlink.
  pub fn is_forbidden<P: AsRef<Path>>(&self, path: P) -> bool {
    let path = try_resolve_symlink_and_canonicalize(path);

    if let Ok(path) = path {
      let path: PathBuf = path.components().collect();
      self
        .forbidden_patterns
        .lock()
        .unwrap()
        .iter()
        .any(|p| p.matches_path_with(&path, self.match_options))
    } else {
      true
    }
  }
}

fn try_resolve_symlink_and_canonicalize<P: AsRef<Path>>(path: P) -> crate::Result<PathBuf> {
  let path = path.as_ref();
  let path = if path.is_symlink() {
    std::fs::read_link(path)?
  } else {
    path.to_path_buf()
  };
  if !path.exists() {
    crate::Result::Ok(path)
  } else {
    std::fs::canonicalize(path).map_err(Into::into)
  }
}

fn escaped_pattern(p: &str) -> Result<Pattern, glob::PatternError> {
  Pattern::new(&glob::Pattern::escape(p))
}

fn escaped_pattern_with(p: &str, append: &str) -> Result<Pattern, glob::PatternError> {
  if p.ends_with(MAIN_SEPARATOR) {
    Pattern::new(&format!("{}{append}", glob::Pattern::escape(p)))
  } else {
    Pattern::new(&format!(
      "{}{}{append}",
      glob::Pattern::escape(p),
      MAIN_SEPARATOR
    ))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use glob::Pattern;

  use super::{push_pattern, Scope};

  fn new_scope() -> Scope {
    Scope {
      allowed_patterns: Default::default(),
      forbidden_patterns: Default::default(),
      event_listeners: Default::default(),
      next_event_id: Default::default(),
      match_options: glob::MatchOptions {
        // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
        // see: <https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5>
        require_literal_separator: true,
        // dotfiles are not supposed to be exposed by default on unix
        #[cfg(unix)]
        require_literal_leading_dot: true,
        #[cfg(windows)]
        require_literal_leading_dot: false,
        ..Default::default()
      },
    }
  }

  #[test]
  fn path_is_escaped() {
    let scope = new_scope();
    #[cfg(unix)]
    {
      scope.allow_directory("/home/tauri/**", false).unwrap();
      assert!(scope.is_allowed("/home/tauri/**"));
      assert!(scope.is_allowed("/home/tauri/**/file"));
      assert!(!scope.is_allowed("/home/tauri/anyfile"));
    }
    #[cfg(windows)]
    {
      scope.allow_directory("C:\\home\\tauri\\**", false).unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(scope.is_allowed("C:\\home\\tauri\\**\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\anyfile"));
    }

    let scope = new_scope();
    #[cfg(unix)]
    {
      scope.allow_file("/home/tauri/**").unwrap();
      assert!(scope.is_allowed("/home/tauri/**"));
      assert!(!scope.is_allowed("/home/tauri/**/file"));
      assert!(!scope.is_allowed("/home/tauri/anyfile"));
    }
    #[cfg(windows)]
    {
      scope.allow_file("C:\\home\\tauri\\**").unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\**\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\anyfile"));
    }

    let scope = new_scope();
    #[cfg(unix)]
    {
      scope.allow_directory("/home/tauri", true).unwrap();
      scope.forbid_directory("/home/tauri/**", false).unwrap();
      assert!(!scope.is_allowed("/home/tauri/**"));
      assert!(!scope.is_allowed("/home/tauri/**/file"));
      assert!(scope.is_allowed("/home/tauri/**/inner/file"));
      assert!(scope.is_allowed("/home/tauri/inner/folder/anyfile"));
      assert!(scope.is_allowed("/home/tauri/anyfile"));
    }
    #[cfg(windows)]
    {
      scope.allow_directory("C:\\home\\tauri", true).unwrap();
      scope
        .forbid_directory("C:\\home\\tauri\\**", false)
        .unwrap();
      assert!(!scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\**\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\**\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\inner\\folder\\anyfile"));
      assert!(scope.is_allowed("C:\\home\\tauri\\anyfile"));
    }

    let scope = new_scope();
    #[cfg(unix)]
    {
      scope.allow_directory("/home/tauri", true).unwrap();
      scope.forbid_file("/home/tauri/**").unwrap();
      assert!(!scope.is_allowed("/home/tauri/**"));
      assert!(scope.is_allowed("/home/tauri/**/file"));
      assert!(scope.is_allowed("/home/tauri/**/inner/file"));
      assert!(scope.is_allowed("/home/tauri/anyfile"));
    }
    #[cfg(windows)]
    {
      scope.allow_directory("C:\\home\\tauri", true).unwrap();
      scope.forbid_file("C:\\home\\tauri\\**").unwrap();
      assert!(!scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(scope.is_allowed("C:\\home\\tauri\\**\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\**\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\anyfile"));
    }

    let scope = new_scope();
    #[cfg(unix)]
    {
      scope.allow_directory("/home/tauri", false).unwrap();
      assert!(scope.is_allowed("/home/tauri/**"));
      assert!(!scope.is_allowed("/home/tauri/**/file"));
      assert!(!scope.is_allowed("/home/tauri/**/inner/file"));
      assert!(scope.is_allowed("/home/tauri/anyfile"));
    }
    #[cfg(windows)]
    {
      scope.allow_directory("C:\\home\\tauri", false).unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\**\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\**\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\anyfile"));
    }
  }

  #[cfg(windows)]
  #[test]
  fn windows_root_paths() {
    let scope = new_scope();
    {
      // UNC network path
      scope.allow_directory("\\\\localhost\\c$", true).unwrap();
      assert!(scope.is_allowed("\\\\localhost\\c$"));
      assert!(scope.is_allowed("\\\\localhost\\c$\\Windows"));
      assert!(scope.is_allowed("\\\\localhost\\c$\\NonExistentFile"));
      assert!(!scope.is_allowed("\\\\localhost\\d$"));
      assert!(!scope.is_allowed("\\\\OtherServer\\Share"));
    }

    let scope = new_scope();
    {
      // Verbatim UNC network path
      scope
        .allow_directory("\\\\?\\UNC\\localhost\\c$", true)
        .unwrap();
      assert!(scope.is_allowed("\\\\localhost\\c$"));
      assert!(scope.is_allowed("\\\\localhost\\c$\\Windows"));
      assert!(scope.is_allowed("\\\\?\\UNC\\localhost\\c$\\Windows\\NonExistentFile"));
      // A non-existent file cannot be canonicalized to a verbatim UNC path, so this will fail to match
      assert!(!scope.is_allowed("\\\\localhost\\c$\\Windows\\NonExistentFile"));
      assert!(!scope.is_allowed("\\\\localhost\\d$"));
      assert!(!scope.is_allowed("\\\\OtherServer\\Share"));
    }

    let scope = new_scope();
    {
      // Device namespace
      scope.allow_file("\\\\.\\COM1").unwrap();
      assert!(scope.is_allowed("\\\\.\\COM1"));
      assert!(!scope.is_allowed("\\\\.\\COM2"));
    }

    let scope = new_scope();
    {
      // Disk root
      scope.allow_directory("C:\\", true).unwrap();
      assert!(scope.is_allowed("C:\\Windows"));
      assert!(scope.is_allowed("C:\\Windows\\system.ini"));
      assert!(scope.is_allowed("C:\\NonExistentFile"));
      assert!(!scope.is_allowed("D:\\home"));
    }

    let scope = new_scope();
    {
      // Verbatim disk root
      scope.allow_directory("\\\\?\\C:\\", true).unwrap();
      assert!(scope.is_allowed("C:\\Windows"));
      assert!(scope.is_allowed("C:\\Windows\\system.ini"));
      assert!(scope.is_allowed("C:\\NonExistentFile"));
      assert!(!scope.is_allowed("D:\\home"));
    }

    let scope = new_scope();
    {
      // Verbatim path
      scope.allow_file("\\\\?\\anyfile").unwrap();
      assert!(scope.is_allowed("\\\\?\\anyfile"));
      assert!(!scope.is_allowed("\\\\?\\otherfile"));
    }

    let cwd = std::env::current_dir().unwrap();
    let disk = {
      let std::path::Component::Prefix(prefix) = cwd.components().next().unwrap() else {
        panic!("Expected current dir to start with a prefix");
      };
      assert!(
        matches!(prefix.kind(), std::path::Prefix::Disk(_)),
        "Expected current dir to be on a disk drive"
      );
      prefix.as_os_str().to_string_lossy()
    };

    let scope = new_scope();
    {
      // Disk
      scope.allow_directory(&*disk, true).unwrap();
      assert!(scope.is_allowed(format!("{}Cargo.toml", disk)));
      assert!(scope.is_allowed(cwd.join("Cargo.toml")));
      assert!(!scope.is_allowed("C:\\Windows"));
      assert!(!scope.is_allowed("Q:Cargo.toml"));
    }

    let scope = new_scope();
    {
      // Verbatim disk
      scope
        .allow_directory(format!("\\\\?\\{}", disk), true)
        .unwrap();
      assert!(scope.is_allowed(format!("{}Cargo.toml", disk)));
      assert!(scope.is_allowed(cwd.join("Cargo.toml")));
      assert!(!scope.is_allowed("C:\\Windows"));
      assert!(!scope.is_allowed("Q:Cargo.toml"));
    }
  }

  #[test]
  fn push_pattern_generated_paths() {
    macro_rules! assert_pattern {
      ($patterns:ident, $pattern:literal) => {
        assert!($patterns.contains(&Pattern::new($pattern).unwrap()))
      };
    }

    let mut patterns = HashSet::new();

    #[cfg(not(windows))]
    {
      push_pattern(&mut patterns, "/path/to/dir/", Pattern::new).expect("failed to push pattern");
      push_pattern(&mut patterns, "/path/to/dir/**", Pattern::new).expect("failed to push pattern");

      assert_pattern!(patterns, "/path/to/dir");
      assert_pattern!(patterns, "/path/to/dir/**");
    }

    #[cfg(windows)]
    {
      push_pattern(&mut patterns, "C:\\path\\to\\dir", Pattern::new)
        .expect("failed to push pattern");
      push_pattern(&mut patterns, "C:\\path\\to\\dir\\**", Pattern::new)
        .expect("failed to push pattern");

      assert_pattern!(patterns, "C:\\path\\to\\dir");
      assert_pattern!(patterns, "C:\\path\\to\\dir\\**");
      assert_pattern!(patterns, "\\\\?\\C:\\path\\to\\dir");
      assert_pattern!(patterns, "\\\\?\\C:\\path\\to\\dir\\**");
    }
  }
}
