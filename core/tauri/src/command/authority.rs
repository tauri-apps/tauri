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
    Self {
      allowed_commands: acl.allowed_commands,
      denied_commands: acl.denied_commands,
      scope_manager: ScopeManager {
        raw: acl.scope,
        cache: <TypeMap![Send + Sync]>::new(),
      },
    }
  }

  /// Checks if the given IPC execution is allowed and returns the [`ResolvedCommand`] if it is.
  pub fn resolve_access(
    &self,
    command: &str,
    window: &String,
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
        .filter(|allowed| allowed.windows.contains(window))
    }
  }
}

#[derive(Debug)]
struct CommandScope<T: Debug + DeserializeOwned + Send + Sync + 'static> {
  allow: Vec<T>,
  deny: Vec<T>,
}

/// Access scope for a command that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct AccessScope<'a, T: Debug + DeserializeOwned + Send + Sync + 'static>(
  &'a CommandScope<T>,
);

impl<'a, T: Debug + DeserializeOwned + Send + Sync + 'static> AccessScope<'a, T> {
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
  for AccessScope<'a, T>
{
  /// Grabs the [`Window`] from the [`CommandItem`] and returns the associated [`Scope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    command
      .acl
      .as_ref()
      .and_then(|resolved| {
        command
          .message
          .window
          .manager
          .runtime_authority
          .scope_manager
          .get_typed(&resolved.scope)
          .map(AccessScope)
      })
      .ok_or_else(|| InvokeError::from_anyhow(anyhow::anyhow!("scope not found")))
  }
}

#[derive(Debug, Default)]
pub struct ScopeManager {
  raw: BTreeMap<ScopeKey, ResolvedScope>,
  cache: TypeMap![Send + Sync],
}

impl ScopeManager {
  fn get_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
    key: &ScopeKey,
  ) -> Option<&CommandScope<T>> {
    match self.cache.try_get() {
      cached @ Some(_) => cached,
      None => match self.raw.get(key).map(|r| {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        for allowed in &r.allow {
          allow.push(allowed.deserialize().unwrap());
        }
        for denied in &r.deny {
          deny.push(denied.deserialize().unwrap());
        }

        CommandScope { allow, deny }
      }) {
        None => None,
        Some(value) => {
          let _ = self.cache.set(value);
          self.cache.try_get()
        }
      },
    }
  }
}
