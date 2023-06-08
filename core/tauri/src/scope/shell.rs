// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(shell_execute, shell_sidecar))]
use crate::api::process::Command;
#[cfg(feature = "shell-open-api")]
use crate::api::shell::Program;

use regex::Regex;
use tauri_utils::{config::Config, Env, PackageInfo};
use uuid::Uuid;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

/// Scope change event.
#[derive(Debug, Clone)]
pub enum Event {
  /// A command has been allowed.
  CommandAllowed {
    /// The command key.
    name: String,
    /// The command scope definition.
    scope: ScopeAllowedCommand,
  },
}

type EventListener = Box<dyn Fn(&Event) + Send>;

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
  command: PathBuf,

  /// The arguments the command is allowed to be called with.
  args: Option<Vec<ScopeAllowedArg>>,

  /// If this command is a sidecar command.
  sidecar: bool,
}

impl ScopeAllowedCommand {
  /// The command of this scope.
  pub fn command(&self) -> &Path {
    &self.command
  }

  /// Whether the command is a sidecar or not.
  pub fn sidecar(&self) -> bool {
    self.sidecar
  }

  /// Whether this scope allows any CLI argument list or not.
  pub fn allows_any_args(&self) -> bool {
    self.args.is_none()
  }

  /// The CLI argument validation of this scope.
  pub fn args(&self) -> Option<&[ScopeAllowedArg]> {
    self.args.as_deref()
  }
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

/// A builder for [`ScopeAllowedCommand`].
#[derive(Debug)]
pub struct ScopeAllowedCommandBuilder(ScopeAllowedCommand);

impl ScopeAllowedCommandBuilder {
  /// Prepares a new command to allow on the shell scope.
  ///
  /// By default CLI arguments are not allowed. Use [`Self::arg`] or [`Self::allow_any_args`] if you are going to use them.
  pub fn new<P: AsRef<Path>>(command: P) -> Self {
    Self(ScopeAllowedCommand {
      command: command.as_ref().to_path_buf(),
      args: Some(Vec::new()),
      sidecar: false,
    })
  }

  /// Prepares a new sidecar to allow on the shell scope.
  ///
  /// By default CLI arguments are not allowed. Use [`Self::arg`] or [`Self::allow_any_args`] if you are going to use them.
  pub fn sidecar<P: AsRef<Path>>(command: P) -> Self {
    Self(ScopeAllowedCommand {
      command: command.as_ref().to_path_buf(),
      args: Some(Vec::new()),
      sidecar: true,
    })
  }

  /// Disable CLI argument validation. If possible, prefer [`Self::arg`] for security.
  pub fn allow_any_args(mut self) -> Self {
    self.0.args = None;
    self
  }

  /// Appends an argument to the command.
  pub fn arg(mut self, arg: ScopeAllowedArg) -> Self {
    self.0.args.get_or_insert_with(Default::default).push(arg);
    self
  }

  /// Builds the [`ScopeAllowedCommand`] to use on [`Scope::allow`].
  pub fn build(self) -> ScopeAllowedCommand {
    self.0
  }
}

/// Scope for shell access.
#[derive(Clone)]
pub struct Scope {
  config: Arc<Mutex<ScopeConfig>>,
  event_listeners: Arc<Mutex<HashMap<Uuid, EventListener>>>,
}

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
  pub(crate) fn new(
    config: &Config,
    package_info: &PackageInfo,
    env: &Env,
    mut scope: ScopeConfig,
  ) -> Self {
    for cmd in scope.scopes.values_mut() {
      if let Ok(path) = crate::api::path::parse(config, package_info, env, &cmd.command) {
        cmd.command = path;
      }
    }
    Self {
      config: Arc::new(Mutex::new(scope)),
      event_listeners: Default::default(),
    }
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

  /// Allow a command to be executed.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, scope::ShellScopeAllowedCommandBuilder};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     app.shell_scope().allow("java", ShellScopeAllowedCommandBuilder::new("java").build());
  ///     app.shell_scope().allow("server-sidecar", ShellScopeAllowedCommandBuilder::sidecar("server").build());
  ///     Ok(())
  ///   });
  /// ```
  pub fn allow<S: Into<String>>(&self, name: S, command: ScopeAllowedCommand) {
    let name = name.into();
    self
      .config
      .lock()
      .unwrap()
      .scopes
      .insert(name.clone(), command.clone());

    self.trigger(Event::CommandAllowed {
      name,
      scope: command,
    });
  }

  /// Validates argument inputs and creates a Tauri sidecar [`Command`].
  #[cfg(shell_sidecar)]
  pub fn prepare_sidecar(
    &self,
    command_name: &str,
    command_script: &str,
    args: ExecuteArgs,
  ) -> Result<Command, ScopeError> {
    self._prepare(command_name, args, Some(command_script))
  }

  /// Validates argument inputs and creates a Tauri [`Command`].
  #[cfg(shell_execute)]
  pub fn prepare(&self, command_name: &str, args: ExecuteArgs) -> Result<Command, ScopeError> {
    self._prepare(command_name, args, None)
  }

  /// Validates argument inputs and creates a Tauri [`Command`].
  #[cfg(any(shell_execute, shell_sidecar))]
  pub fn _prepare(
    &self,
    command_name: &str,
    args: ExecuteArgs,
    sidecar: Option<&str>,
  ) -> Result<Command, ScopeError> {
    let scope = self.config.lock().unwrap();
    let command = match scope.scopes.get(command_name) {
      Some(command) => command,
      None => return Err(ScopeError::NotFound(command_name.into())),
    };

    if command.sidecar != sidecar.is_some() {
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

    let command_s = sidecar
      .map(|s| {
        std::path::PathBuf::from(s)
          .components()
          .last()
          .unwrap()
          .as_os_str()
          .to_string_lossy()
          .into_owned()
      })
      .unwrap_or_else(|| command.command.to_string_lossy().into_owned());
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
  /// defaults to `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`.
  #[cfg(feature = "shell-open-api")]
  pub fn open(&self, path: &str, with: Option<Program>) -> Result<(), ScopeError> {
    // ensure we pass validation if the configuration has one
    if let Some(regex) = &self.config.lock().unwrap().open {
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
      Some(program) => ::open::with(path, program),
      None => ::open::that(path),
    }
    .map_err(Into::into)
  }
}
