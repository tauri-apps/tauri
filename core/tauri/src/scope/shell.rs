// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(shell_execute, shell_sidecar))]
use crate::api::process::Command;
#[cfg(feature = "shell-open-api")]
use crate::api::shell::Program;

use regex::Regex;

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
}

impl ExecuteArgs {
  /// Whether the argument list is empty or not.
  pub fn is_empty(&self) -> bool {
    match self {
      Self::None => true,
      Self::Single(s) if s.is_empty() => true,
      Self::List(l) => l.is_empty(),
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

/// Shell scope configuration.
#[derive(Debug, Clone)]
pub struct ScopeConfig {
  /// The validation regex that `shell > open` paths must match against.
  pub open: Option<Regex>,

  /// All allowed commands, using their unique command name as the keys.
  pub scopes: HashMap<String, ScopeAllowedCommand>,
}

/// A configured scoped shell command.
#[derive(Debug, Clone)]
pub struct ScopeAllowedCommand {
  /// The shell command to be called.
  pub command: std::path::PathBuf,

  /// The arguments the command is allowed to be called with.
  pub args: Option<Vec<ScopeAllowedArg>>,

  /// If this command is a sidecar command.
  pub sidecar: bool,
}

/// A configured argument to a scoped shell command.
#[derive(Debug, Clone)]
pub enum ScopeAllowedArg {
  /// A non-configurable argument.
  Fixed(String),

  /// An argument with a value to be evaluated at runtime, must pass a regex validation.
  Var {
    /// The validation that the variable value must pass in order to be called.
    validator: Regex,
  },
}

impl ScopeAllowedArg {
  /// If the argument is fixed.
  pub fn is_fixed(&self) -> bool {
    matches!(self, Self::Fixed(_))
  }

  /// If the argument is a variable value.
  pub fn is_var(&self) -> bool {
    matches!(self, Self::Var { .. })
  }
}

/// Scope for filesystem access.
#[derive(Clone)]
pub struct Scope(ScopeConfig);

/// All errors that can happen while validating a scoped command.
#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
  /// At least one argument did not pass input validation.
  #[cfg(any(shell_execute, shell_sidecar))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  #[error("The scoped command was called with the improper sidecar flag set")]
  BadSidecarFlag,

  /// The sidecar program validated but failed to find the sidecar path.
  ///
  /// Note: This can be called on `shell-execute` feature too due to [`Scope::prepare`] checking if
  /// it's a sidecar from the config.
  #[cfg(any(shell_execute, shell_sidecar))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  #[error(
    "The scoped sidecar command was validated, but failed to create the path to the command: {0}"
  )]
  Sidecar(crate::Error),

  /// The named command was not found in the scoped config.
  #[error("Scoped command {0} not found")]
  #[cfg(any(shell_execute, shell_sidecar))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  NotFound(String),

  /// A command variable has no value set in the arguments.
  #[error(
    "Scoped command argument at position {0} must match regex validation {1} but it was not found"
  )]
  #[cfg(any(shell_execute, shell_sidecar))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  MissingVar(usize, String),

  /// At least one argument did not pass input validation.
  #[cfg(shell_scope)]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-open")))
  )]
  #[error("Scoped command argument at position {index} was found, but failed regex validation {validation}")]
  Validation {
    /// Index of the variable.
    index: usize,

    /// Regex that the variable value failed to match.
    validation: String,
  },

  /// The format of the passed input does not match the expected shape.
  ///
  /// This can happen from passing a string or array of strings to a command that is expecting
  /// named variables, and vice-versa.
  #[cfg(any(shell_execute, shell_sidecar))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  #[error("Scoped command {0} received arguments in an unexpected format")]
  InvalidInput(String),

  /// A generic IO error that occurs while executing specified shell commands.
  #[cfg(shell_scope)]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "shell-execute", feature = "shell-sidecar")))
  )]
  #[error("Scoped shell IO error: {0}")]
  Io(#[from] std::io::Error),
}

impl Scope {
  /// Creates a new shell scope.
  pub fn new(scope: ScopeConfig) -> Self {
    Self(scope)
  }

  /// Validates argument inputs and creates a Tauri [`Command`].
  #[cfg(any(shell_execute, shell_sidecar))]
  pub fn prepare(
    &self,
    command_name: &str,
    args: ExecuteArgs,
    sidecar: bool,
  ) -> Result<Command, ScopeError> {
    let command = match self.0.scopes.get(command_name) {
      Some(command) => command,
      None => return Err(ScopeError::NotFound(command_name.into())),
    };

    if command.sidecar != sidecar {
      return Err(ScopeError::BadSidecarFlag);
    }

    let args = match (&command.args, args) {
      (None, ExecuteArgs::None) => Ok(vec![]),
      (None, ExecuteArgs::List(list)) => Ok(list),
      (None, ExecuteArgs::Single(string)) => Ok(vec![string]),
      (Some(list), ExecuteArgs::List(args)) => list
        .iter()
        .enumerate()
        .map(|(i, arg)| match arg {
          ScopeAllowedArg::Fixed(fixed) => Ok(fixed.to_string()),
          ScopeAllowedArg::Var { validator } => {
            let value = args
              .get(i)
              .ok_or_else(|| ScopeError::MissingVar(i, validator.to_string()))?
              .to_string();
            if validator.is_match(&value) {
              Ok(value)
            } else {
              Err(ScopeError::Validation {
                index: i,
                validation: validator.to_string(),
              })
            }
          }
        })
        .collect(),
      (Some(list), arg) if arg.is_empty() && list.iter().all(ScopeAllowedArg::is_fixed) => list
        .iter()
        .map(|arg| match arg {
          ScopeAllowedArg::Fixed(fixed) => Ok(fixed.to_string()),
          _ => unreachable!(),
        })
        .collect(),
      (Some(list), _) if list.is_empty() => Err(ScopeError::InvalidInput(command_name.into())),
      (Some(_), _) => Err(ScopeError::InvalidInput(command_name.into())),
    }?;

    let command_s = command.command.to_string_lossy();
    let command = if command.sidecar {
      Command::new_sidecar(command_s).map_err(ScopeError::Sidecar)?
    } else {
      Command::new(command_s)
    };

    Ok(command.args(args))
  }

  /// Open a path in the default (or specified) browser.
  ///
  /// The path is validated against the `tauri > allowlist > shell > open` validation regex, which
  /// defaults to `^https?://`.
  #[cfg(feature = "shell-open-api")]
  pub fn open(&self, path: &str, with: Option<Program>) -> Result<(), ScopeError> {
    // ensure we pass validation if the configuration has one
    if let Some(regex) = &self.0.open {
      if !regex.is_match(path) {
        return Err(ScopeError::Validation {
          index: 0,
          validation: regex.as_str().into(),
        });
      }
    }

    // The prevention of argument escaping is handled by the usage of std::process::Command::arg by
    // the `open` dependency. This behavior should be re-confirmed during upgrades of `open`.
    match with.map(Program::name) {
      Some(program) => ::open::with(&path, program),
      None => ::open::that(&path),
    }
    .map_err(Into::into)
  }
}
