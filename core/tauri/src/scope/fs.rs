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
    let (allowed_patterns, forbidden_patterns) = Self::patterns(config, package_info, env, scope)?;

    let require_literal_leading_dot = match scope {
      FsAllowlistScope::Scope {
        require_literal_leading_dot: Some(require),
        ..
      } => *require,
      // dotfiles are not supposed to be exposed by default on unix
      _ => cfg!(unix),
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

  /// This removes any allowed or forbidden paths by resetting
  /// the scope to its initial state which matches the scope defined in the configuration file.
  ///
  /// Returns the removed allowed and forbidden patterns.
  pub fn reset(&self) -> ResetScope {
    let mut allowed_patterns = self.allowed_patterns.lock().unwrap();
    let mut forbidden_patterns = self.forbidden_patterns.lock().unwrap();
    let allowed_diff = allowed_patterns
      .difference(&self.default_allowed)
      .cloned()
      .collect();
    let forbidden_diff = forbidden_patterns
      .difference(&self.default_forbidden)
      .cloned()
      .collect();
    *allowed_patterns = self.default_allowed.clone();
    *forbidden_patterns = self.default_forbidden.clone();

    ResetScope {
      allowed_patterns: allowed_diff,
      forbidden_patterns: forbidden_diff,
    }
  }

  /// Extend the list of allowed patterns with the given iterator.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::Manager;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let scope = app.fs_scope();
  ///     let reset = scope.reset();
  ///     scope.extend_allowed(reset.allowed_patterns().into_iter().cloned());
  ///     Ok(())
  ///   });
  /// ```
  pub fn extend_allowed<I: IntoIterator<Item = Pattern>>(&self, patterns: I) {
    self.allowed_patterns.lock().unwrap().extend(patterns);
  }

  /// Extend the list of forbidden patterns with the given iterator.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::Manager;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let scope = app.fs_scope();
  ///     let reset = scope.reset();
  ///     scope.extend_forbidden(reset.forbidden_patterns().into_iter().cloned());
  ///     Ok(())
  ///   });
  /// ```
  pub fn extend_forbidden<I: IntoIterator<Item = Pattern>>(&self, patterns: I) {
    self.forbidden_patterns.lock().unwrap().extend(patterns);
  }
}

/// Contains the scope information that was reset after calling [`reset`](`Scope#method.reset`).
pub struct ResetScope {
  allowed_patterns: HashSet<Pattern>,
  forbidden_patterns: HashSet<Pattern>,
}

impl ResetScope {
  /// The list of allowed patterns that were reset.
  pub fn allowed_patterns(&self) -> &HashSet<Pattern> {
    &self.allowed_patterns
  }

  /// The list of forbidden patterns that were reset.
  pub fn forbidden_patterns(&self) -> &HashSet<Pattern> {
    &self.forbidden_patterns
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

  use super::{escaped_pattern, escaped_pattern_with, Scope};
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
        require_literal_leading_dot: cfg!(unix),
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
    let home = PathBuf::from("/home/tauri");
    #[cfg(windows)]
    let home = PathBuf::from("C:\\home\\tauri");

    scope.allow_directory(home.join("**"), false).unwrap();
    assert!(scope.is_allowed(home.join("**")));
    assert!(scope.is_allowed(home.join("**").join("file")));
    assert!(!scope.is_allowed(home.join("anyfile")));

    let scope = new_scope();
    scope.allow_file(home.join("**")).unwrap();
    assert!(scope.is_allowed(home.join("**")));
    assert!(!scope.is_allowed(home.join("**").join("file")));
    assert!(!scope.is_allowed(home.join("anyfile")));

    let scope = new_scope();

    scope.allow_directory(&home, true).unwrap();
    scope.forbid_directory(home.join("**"), false).unwrap();
    assert!(!scope.is_allowed(home.join("**")));
    assert!(!scope.is_allowed(home.join("**").join("file")));
    assert!(scope.is_allowed(home.join("**").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("inner").join("folder").join("anyfile")));
    assert!(scope.is_allowed(home.join("anyfile")));

    let scope = new_scope();

    scope.allow_directory(&home, true).unwrap();
    scope.forbid_file(home.join("**")).unwrap();
    assert!(!scope.is_allowed(home.join("**")));
    assert!(scope.is_allowed(home.join("**").join("file")));
    assert!(scope.is_allowed(home.join("**").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("anyfile")));

    let scope = new_scope();

    scope.allow_directory(&home, false).unwrap();
    assert!(scope.is_allowed(home.join("**")));
    assert!(!scope.is_allowed(home.join("**").join("file")));
    assert!(!scope.is_allowed(home.join("**").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("anyfile")));
  }

  #[test]
  fn reset() {
    let app = mock_app();

    #[cfg(unix)]
    let home = PathBuf::from("/home/tauri");
    #[cfg(windows)]
    let home = PathBuf::from("C:\\home\\tauri");

    let allowlist = FsAllowlistScope::Scope {
      allow: vec![home.join("allowed"), home.join("allowed").join("**")],
      deny: vec![home.join("forbidden"), home.join("forbidden/**")],
      require_literal_leading_dot: Some(cfg!(unix)),
    };

    let scope = new_scope_from_allowlist(&app, &allowlist);

    assert!(scope.is_allowed(home.join("allowed")));
    assert!(scope.is_allowed(home.join("allowed").join("file")));
    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("allowed").join("**")));

    assert!(!scope.is_allowed(home.join("forbidden")));
    assert!(!scope.is_allowed(home.join("forbidden").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("**")));

    let scope = new_scope_from_allowlist(&app, &allowlist);

    scope.allow_directory(&home, true).unwrap();
    assert!(scope.is_allowed(home.join("inner").join("file")));
    assert!(scope.is_allowed(home.join("**")));
    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));

    let reset = scope.reset();
    assert_eq!(
      reset.allowed_patterns,
      vec![
        escaped_pattern(&home.to_string_lossy()).unwrap(),
        escaped_pattern_with(&home.to_string_lossy(), "**").unwrap()
      ]
      .into_iter()
      .collect()
    );
    assert_eq!(reset.forbidden_patterns, vec![].into_iter().collect());

    assert!(!scope.is_allowed(home.join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("**")));
    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));

    scope.allow_file(home.join("allowed_file")).unwrap();
    scope.allow_directory(home.join("workspace"), true).unwrap();
    scope.forbid_file(home.join("forbidden_file")).unwrap();
    scope.forbid_directory(home.join("ignored"), true).unwrap();
    assert!(scope.is_allowed(home.join("allowed_file")));
    assert!(scope.is_allowed(home.join("workspace").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden_file")));
    assert!(!scope.is_allowed(home.join("ignored").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));

    let reset = scope.reset();
    assert_eq!(
      reset.allowed_patterns,
      vec![
        escaped_pattern(&home.join("allowed_file").to_string_lossy()).unwrap(),
        escaped_pattern(&home.join("workspace").to_string_lossy()).unwrap(),
        escaped_pattern_with(&home.join("workspace").to_string_lossy(), "**").unwrap()
      ]
      .into_iter()
      .collect()
    );
    assert_eq!(
      reset.forbidden_patterns,
      vec![
        escaped_pattern(&home.join("forbidden_file").to_string_lossy()).unwrap(),
        escaped_pattern(&home.join("ignored").to_string_lossy()).unwrap(),
        escaped_pattern_with(&home.join("ignored").to_string_lossy(), "**").unwrap()
      ]
      .into_iter()
      .collect()
    );

    assert!(!scope.is_allowed(home.join("allowed_file")));
    assert!(!scope.is_allowed(home.join("workspace").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden_file")));
    assert!(!scope.is_allowed(home.join("ignored").join("inner").join("file")));
    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));

    scope
      .forbid_directory(home.join("allowed").join("inner"), true)
      .unwrap();
    scope
      .allow_directory(home.join("forbidden").join("inner"), true)
      .unwrap();
    assert!(!scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));

    let reset = scope.reset();
    assert_eq!(
      reset.allowed_patterns,
      vec![
        escaped_pattern(&home.join("forbidden").join("inner").to_string_lossy()).unwrap(),
        escaped_pattern_with(
          &home.join("forbidden").join("inner").to_string_lossy(),
          "**"
        )
        .unwrap()
      ]
      .into_iter()
      .collect()
    );
    assert_eq!(
      reset.forbidden_patterns,
      vec![
        escaped_pattern(&home.join("allowed").join("inner").to_string_lossy()).unwrap(),
        escaped_pattern_with(&home.join("allowed").join("inner").to_string_lossy(), "**").unwrap()
      ]
      .into_iter()
      .collect()
    );

    assert!(scope.is_allowed(home.join("allowed").join("inner").join("file")));
    assert!(!scope.is_allowed(home.join("forbidden").join("inner").join("file")));
  }
}
