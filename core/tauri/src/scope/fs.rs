// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{HashMap, HashSet},
  fmt,
  path::{Path, PathBuf, MAIN_SEPARATOR},
  sync::{Arc, Mutex},
};

pub use glob::Pattern;
use tauri_utils::{
  config::{Config, FsAllowlistScope},
  Env, PackageInfo,
};
use uuid::Uuid;

use crate::api::path::parse as parse_path;

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
  event_listeners: Arc<Mutex<HashMap<Uuid, EventListener>>>,
  match_options: glob::MatchOptions,
  default_allowed: HashSet<Pattern>,
  default_forbidden: HashSet<Pattern>,
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
  #[cfg(windows)]
  {
    if let Ok(p) = std::fs::canonicalize(&path) {
      list.insert(f(&p.to_string_lossy())?);
    } else {
      list.insert(f(&format!("\\\\?\\{}", path.display()))?);
    }
  }
  Ok(())
}

impl Scope {
  /// Gets allowed and forbidden patterns from a `FsAllowlistScope` configuration
  fn patterns(
    config: &Config,
    package_info: &PackageInfo,
    env: &Env,
    scope: &FsAllowlistScope,
  ) -> crate::Result<(HashSet<Pattern>, HashSet<Pattern>)> {
    let mut allowed_patterns = HashSet::new();
    for path in scope.allowed_paths() {
      if let Ok(path) = parse_path(config, package_info, env, path) {
        push_pattern(&mut allowed_patterns, path, Pattern::new)?;
      }
    }

    let mut forbidden_patterns = HashSet::new();
    if let Some(forbidden_paths) = scope.forbidden_paths() {
      for path in forbidden_paths {
        if let Ok(path) = parse_path(config, package_info, env, path) {
          push_pattern(&mut forbidden_patterns, path, Pattern::new)?;
        }
      }
    }

    Ok((allowed_patterns, forbidden_patterns))
  }

  /// Creates a new scope from a `FsAllowlistScope` configuration.
  #[allow(unused)]
  pub(crate) fn for_fs_api(
    config: &Config,
    package_info: &PackageInfo,
    env: &Env,
    scope: &FsAllowlistScope,
  ) -> crate::Result<Self> {
    let (allowed_patterns, forbidden_patterns) = Scope::patterns(config, package_info, env, scope)?;

    let require_literal_leading_dot = match scope {
      FsAllowlistScope::Scope {
        require_literal_leading_dot: Some(require),
        ..
      } => *require,
      // dotfiles are not supposed to be exposed by default on unix
      #[cfg(unix)]
      _ => true,
      #[cfg(windows)]
      _ => false,
    };

    let default_allowed = allowed_patterns.clone();
    let default_forbidden = forbidden_patterns.clone();

    Ok(Self {
      allowed_patterns: Arc::new(Mutex::new(allowed_patterns)),
      forbidden_patterns: Arc::new(Mutex::new(forbidden_patterns)),
      event_listeners: Default::default(),
      match_options: glob::MatchOptions {
        // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
        // see: https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5
        require_literal_separator: true,
        require_literal_leading_dot,
        ..Default::default()
      },
      default_allowed,
      default_forbidden,
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
  pub fn listen<F: Fn(&Event) + Send + 'static>(&self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self.event_listeners.lock().unwrap().insert(id, Box::new(f));
    id
  }

  fn trigger(&self, event: Event) {
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
    self.trigger(Event::PathAllowed(path.to_path_buf()));
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
    self.trigger(Event::PathAllowed(path.to_path_buf()));
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
    self.trigger(Event::PathForbidden(path.to_path_buf()));
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
    self.trigger(Event::PathForbidden(path.to_path_buf()));
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

  /// This removes any paths added to allowed and forbidden scope by resetting
  /// to the scope state to it's initial/default configuration
  pub fn reset(&self) -> crate::Result<()> {
    *self.allowed_patterns.lock().as_deref_mut().unwrap() = self.default_allowed.clone();
    *self.forbidden_patterns.lock().as_deref_mut().unwrap() = self.default_forbidden.clone();

    Ok(())
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
  use std::path::PathBuf;

  use super::Scope;
  use crate::{test::mock_app, App};

  use tauri_utils::{config::FsAllowlistScope, Env};

  fn new_scope() -> Scope {
    Scope {
      allowed_patterns: Default::default(),
      forbidden_patterns: Default::default(),
      event_listeners: Default::default(),
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
      default_allowed: Default::default(),
      default_forbidden: Default::default(),
    }
  }

  fn new_scope_from_allowlist<R: crate::Runtime>(app: &App<R>, scope: &FsAllowlistScope) -> Scope {
    let env = Env::default();
    Scope::for_fs_api(&app.config(), app.package_info(), &env, scope).unwrap()
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

  #[test]
  fn reset() {
    let app = mock_app();

    #[cfg(unix)]
    let allowlist = FsAllowlistScope::Scope {
      allow: vec![
        PathBuf::from("/home/tauri/allowed"),
        PathBuf::from("/home/tauri/allowed/**"),
      ],
      deny: vec![
        PathBuf::from("/home/tauri/forbidden"),
        PathBuf::from("/home/tauri/forbidden/**"),
      ],
      require_literal_leading_dot: Some(true),
    };

    #[cfg(windows)]
    let allowlist = FsAllowlistScope::Scope {
      allow: vec![
        PathBuf::from("C:\\home\\tauri\\allowed"),
        PathBuf::from("C:\\home\\tauri\\allowed\\**"),
      ],
      deny: vec![
        PathBuf::from("C:\\home\\tauri\\forbidden"),
        PathBuf::from("C:\\home\\tauri\\forbidden\\**"),
      ],
      require_literal_leading_dot: Some(false),
    };

    let scope = new_scope_from_allowlist(&app, &allowlist);
    #[cfg(unix)]
    {
      assert!(scope.is_allowed("/home/tauri/allowed"));
      assert!(scope.is_allowed("/home/tauri/allowed/file"));
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(scope.is_allowed("/home/tauri/allowed/**"));

      assert!(!scope.is_allowed("/home/tauri/forbidden"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/**"));
    }
    #[cfg(windows)]
    {
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\**"));

      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\**"));
    }

    let scope = new_scope_from_allowlist(&app, &allowlist);
    #[cfg(unix)]
    {
      scope.allow_directory("/home/tauri", true).unwrap();
      assert!(scope.is_allowed("/home/tauri/inner/file"));
      assert!(scope.is_allowed("/home/tauri/**"));
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));

      scope.reset().unwrap();
      assert!(!scope.is_allowed("/home/tauri/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/**"));
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));
    }
    #[cfg(windows)]
    {
      scope.allow_directory("C:\\home\\tauri", true).unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));

      scope.reset().unwrap();
      assert!(!scope.is_allowed("C:\\home\\tauri\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\**"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));
    }

    #[cfg(unix)]
    {
      scope.allow_file("/home/tauri/allowed_file").unwrap();
      scope
        .allow_directory("/home/tauri/workspace", true)
        .unwrap();
      scope.forbid_file("/home/tauri/forbidden_file").unwrap();
      scope.forbid_directory("/home/tauri/ignored", true).unwrap();
      assert!(scope.is_allowed("/home/tauri/allowed_file"));
      assert!(scope.is_allowed("/home/tauri/workspace/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden_file"));
      assert!(!scope.is_allowed("/home/tauri/ignored/inner/file"));
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));

      scope.reset().unwrap();
      assert!(!scope.is_allowed("/home/tauri/allowed_file"));
      assert!(!scope.is_allowed("/home/tauri/workspace/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden_file"));
      assert!(!scope.is_allowed("/home/tauri/ignored/inner/file"));
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));
    }
    #[cfg(windows)]
    {
      scope.allow_file("C:\\home\\tauri\\allowed_file").unwrap();
      scope
        .allow_directory("C:\\home\\tauri\\workspace", true)
        .unwrap();
      scope
        .forbid_file("C:\\home\\tauri\\forbidden_file")
        .unwrap();
      scope
        .forbid_directory("C:\\home\\tauri\\ignored", true)
        .unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed_file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\workspace\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden_file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\ignored\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));

      scope.reset().unwrap();
      assert!(!scope.is_allowed("C:\\home\\tauri\\allowed_file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\workspace\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden_file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\ignored\\inner\\file"));
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));
    }

    #[cfg(unix)]
    {
      scope
        .forbid_directory("/home/tauri/allowed/inner", true)
        .unwrap();
      scope
        .allow_directory("/home/tauri/forbidden/inner", true)
        .unwrap();
      assert!(!scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));

      scope.reset().unwrap();
      assert!(scope.is_allowed("/home/tauri/allowed/inner/file"));
      assert!(!scope.is_allowed("/home/tauri/forbidden/inner/file"));
    }
    #[cfg(windows)]
    {
      scope
        .forbid_directory("C:\\home\\tauri\\allowed\\inner", true)
        .unwrap();
      scope
        .allow_directory("C:\\home\\tauri\\forbidden\\inner", true)
        .unwrap();
      assert!(!scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));

      scope.reset().unwrap();
      assert!(scope.is_allowed("C:\\home\\tauri\\allowed\\inner\\file"));
      assert!(!scope.is_allowed("C:\\home\\tauri\\forbidden\\inner\\file"));
    }
  }
}
