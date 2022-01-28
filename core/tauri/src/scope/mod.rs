// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod fs;
mod http;
mod shell;

pub use self::http::Scope as HttpScope;
pub use fs::Scope as FsScope;
use regex::Regex;
pub use shell::{Scope as ShellScope, ScopeError as ShellScopeError};

use std::collections::HashMap;

/// Allowed representation of `Execute` command arguments.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ExecuteArgs {
  /// No arguments
  None,

  /// A single string argument
  Single(String),

  /// Multiple string arguments
  List(Vec<String>),

  /// Multiple string arguments in a key-value fashion
  Map(HashMap<String, String>),
}

impl ExecuteArgs {
  /// Whether the argument list is empty or not.
  pub fn is_empty(&self) -> bool {
    match self {
      Self::None => true,
      Self::Single(s) if s.is_empty() => true,
      Self::List(l) => l.is_empty(),
      Self::Map(m) => m.is_empty(),
      _ => false,
    }
  }
}

impl From<()> for ExecuteArgs {
  fn from(_: ()) -> Self {
    Self::None
  }
}

impl From<String> for ExecuteArgs {
  fn from(string: String) -> Self {
    Self::Single(string)
  }
}

impl From<Vec<String>> for ExecuteArgs {
  fn from(vec: Vec<String>) -> Self {
    Self::List(vec)
  }
}

impl From<HashMap<String, String>> for ExecuteArgs {
  fn from(map: HashMap<String, String>) -> Self {
    Self::Map(map)
  }
}

/// Shell scope configuration.
#[derive(Debug, Clone)]
pub struct ShellScopeConfig {
  /// The validation regex that `shell > open` paths must match against.
  pub open: Option<Regex>,

  /// All allowed commands, using their unique command name as the keys.
  pub scopes: HashMap<String, ShellScopeAllowedCommand>,
}

/// A configured scoped shell command.
#[derive(Debug, Clone)]
pub struct ShellScopeAllowedCommand {
  /// The shell command to be called.
  pub command: std::path::PathBuf,

  /// The arguments the command is allowed to be called with.
  pub args: Option<Vec<ShellScopeAllowedArg>>,

  /// If this command is a sidecar command.
  pub sidecar: bool,
}

/// A configured argument to a scoped shell command.
#[derive(Debug, Clone)]
pub enum ShellScopeAllowedArg {
  /// A non-configurable argument.
  Fixed(String),

  /// An argument with a value to be evaluated at runtime, optionally must pass a regex validation.
  Var {
    /// The key name of the argument variable
    name: String,

    /// The validation, if set, that the variable value must pass in order to be called.
    validate: Option<regex::Regex>,
  },
}

impl ShellScopeAllowedArg {
  /// If the argument is fixed.
  pub fn is_fixed(&self) -> bool {
    matches!(self, Self::Fixed(_))
  }

  /// If the argument is a variable value.
  pub fn is_var(&self) -> bool {
    matches!(self, Self::Var { .. })
  }
}

pub(crate) struct Scopes {
  pub fs: FsScope,
  #[cfg(protocol_asset)]
  pub asset_protocol: FsScope,
  #[cfg(http_request)]
  pub http: HttpScope,
  pub shell: ShellScope,
}
