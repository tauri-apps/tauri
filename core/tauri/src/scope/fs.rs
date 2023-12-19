// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
  let path: PathBuf = pattern.as_ref().components().collect();
  list.insert(f(&path.to_string_lossy())?);

  let mut path = path;
  let mut checked_path = None;

  // attempt to canonicalize parents in case we have a path like `/data/user/0/appid/**`
  // where `**` obviously does not exist but we need to canonicalize the parent
  //
  // example: given the `/data/user/0/appid/assets/*` path,
  // it's a glob pattern so it won't exist (canonicalize() fails);
  //
  // the second iteration needs to check `/data/user/0/appid/assets` and save the `*` component to append later.
  //
  // if it also does not exist, a third iteration is required to check `/data/user/0/appid`
  // with `assets/*` as the cached value (`checked_path` variable)
  // on Android that gets canonicalized to `/data/data/appid` so the final value will be `/data/data/appid/assets/*`
  // which is the value we want to check when we execute the `is_allowed` function
  let canonicalized = loop {
    if let Ok(path) = path.canonicalize() {
      break Some(if let Some(p) = checked_path {
        path.join(p)
      } else {
        path
      });
    }

    // get the last component of the path as an OsStr
    let last = path.iter().next_back().map(PathBuf::from);
    if let Some(mut p) = last {
      // remove the last component of the path
      // so the next iteration checks its parent
      path.pop();
      // append the already checked path to the last component
      if let Some(checked_path) = &checked_path {
        p.push(checked_path);
      }
      // replace the checked path with the current value
      checked_path.replace(p);
    } else {
      break None;
    }
  };

  if let Some(p) = canonicalized {
    list.insert(f(&p.to_string_lossy())?);
  } else if cfg!(windows) {
    list.insert(f(&format!("\\\\?\\{}", path.display()))?);
  }

  Ok(())
}

impl Scope {
  /// Creates a new scope from a `FsAllowlistScope` configuration.
  #[allow(unused)]
  pub(crate) fn for_fs_api<R: crate::Runtime, M: crate::Manager<R>>(
    manager: &M,
    scope: &tauri_utils::config::FsScope,
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
        // see: https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5
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
  pub fn once<F: FnOnce(&Event) + Send + 'static>(&self, f: F) {
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
  pub fn is_allowed<P: AsRef<Path>>(&self, path: P) -> bool {
    let path = path.as_ref();
    let path = if !path.exists() {
      crate::Result::Ok(path.to_path_buf())
    } else {
      std::fs::canonicalize(path).map_err(Into::into)
    };

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
}

fn escaped_pattern(p: &str) -> Result<Pattern, glob::PatternError> {
  Pattern::new(&glob::Pattern::escape(p))
}

fn escaped_pattern_with(p: &str, append: &str) -> Result<Pattern, glob::PatternError> {
  Pattern::new(&format!(
    "{}{}{append}",
    glob::Pattern::escape(p),
    MAIN_SEPARATOR
  ))
}

#[cfg(test)]
mod tests {
  use super::Scope;

  fn new_scope() -> Scope {
    Scope {
      allowed_patterns: Default::default(),
      forbidden_patterns: Default::default(),
      event_listeners: Default::default(),
      next_event_id: Default::default(),
      match_options: glob::MatchOptions {
        // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
        // see: https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5
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
}
