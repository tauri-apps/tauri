// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use state::TypeMap;

use tauri_utils::acl::{
  resolved::{CommandKey, Resolved, ResolvedCommand, ResolvedScope, ScopeKey},
  ExecutionContext,
};

use crate::{ipc::InvokeError, Runtime};

use super::{CommandArg, CommandItem};

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
  scope_manager: ScopeManager,
}

impl RuntimeAuthority {
  pub(crate) fn new(acl: Resolved) -> Self {
    let command_cache = acl
      .command_scope
      .keys()
      .map(|key| (*key, <TypeMap![Send + Sync]>::new()))
      .collect();
    Self {
      allowed_commands: acl.allowed_commands,
      denied_commands: acl.denied_commands,
      scope_manager: ScopeManager {
        command_scope: acl.command_scope,
        global_scope: acl.global_scope,
        command_cache,
        global_scope_cache: Default::default(),
      },
    }
  }

  /// Checks if the given IPC execution is allowed and returns the [`ResolvedCommand`] if it is.
  pub fn resolve_access(
    &self,
    command: &str,
    window: &str,
    context: ExecutionContext,
  ) -> Option<&ResolvedCommand> {
    let key = CommandKey {
      name: command.into(),
      context,
    };
    if self.denied_commands.contains_key(&key) {
      None
    } else {
      self
        .allowed_commands
        .get(&key)
        .filter(|allowed| allowed.windows.iter().any(|w| w.matches(window)))
    }
  }
}

#[derive(Debug)]
struct ScopeValue<T: Debug + DeserializeOwned + Send + Sync + 'static> {
  allow: Vec<T>,
  deny: Vec<T>,
}

/// Access scope for a command that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct CommandScope<'a, T: Debug + DeserializeOwned + Send + Sync + 'static>(&'a ScopeValue<T>);

impl<'a, T: Debug + DeserializeOwned + Send + Sync + 'static> CommandScope<'a, T> {
  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<T> {
    &self.0.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<T> {
    &self.0.deny
  }
}

impl<'a, R: Runtime, T: Debug + DeserializeOwned + Send + Sync + 'static> CommandArg<'a, R>
  for CommandScope<'a, T>
{
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`CommandScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    command
      .acl
      .as_ref()
      .and_then(|resolved| resolved.scope)
      .and_then(|scope_id| {
        command
          .message
          .window
          .manager
          .runtime_authority
          .scope_manager
          .get_command_scope_typed(&scope_id)
          .map(CommandScope)
      })
      .ok_or_else(|| InvokeError::from_anyhow(anyhow::anyhow!("scope not found")))
  }
}

/// Global access scope that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct GlobalScope<'a, T: Debug + DeserializeOwned + Send + Sync + 'static>(&'a ScopeValue<T>);

impl<'a, T: Debug + DeserializeOwned + Send + Sync + 'static> GlobalScope<'a, T> {
  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<T> {
    &self.0.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<T> {
    &self.0.deny
  }
}

impl<'a, R: Runtime, T: Debug + DeserializeOwned + Send + Sync + 'static> CommandArg<'a, R>
  for GlobalScope<'a, T>
{
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`GlobalScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    let scope = command
      .message
      .window
      .manager
      .runtime_authority
      .scope_manager
      .get_global_scope_typed();
    Ok(GlobalScope(scope))
  }
}

#[derive(Debug)]
pub struct ScopeManager {
  command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  global_scope: ResolvedScope,
  command_cache: BTreeMap<ScopeKey, TypeMap![Send + Sync]>,
  global_scope_cache: TypeMap![Send + Sync],
}

impl ScopeManager {
  fn get_global_scope_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
  ) -> &ScopeValue<T> {
    match self.global_scope_cache.try_get() {
      Some(cached) => cached,
      None => {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        for allowed in &self.global_scope.allow {
          allow.push(allowed.deserialize().unwrap());
        }
        for denied in &self.global_scope.deny {
          deny.push(denied.deserialize().unwrap());
        }

        let scope = ScopeValue { allow, deny };
        let _ = self.global_scope_cache.set(scope);
        self.global_scope_cache.get()
      }
    }
  }

  fn get_command_scope_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
    key: &ScopeKey,
  ) -> Option<&ScopeValue<T>> {
    let cache = self.command_cache.get(key).unwrap();
    match cache.try_get() {
      cached @ Some(_) => cached,
      None => match self.command_scope.get(key).map(|r| {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        for allowed in &r.allow {
          allow.push(allowed.deserialize().unwrap());
        }
        for denied in &r.deny {
          deny.push(denied.deserialize().unwrap());
        }

        ScopeValue { allow, deny }
      }) {
        None => None,
        Some(value) => {
          let _ = cache.set(value);
          cache.try_get()
        }
      },
    }
  }
}
