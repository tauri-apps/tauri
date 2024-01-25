// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

use serde::de::DeserializeOwned;
use state::TypeMap;

use tauri_utils::acl::{
  resolved::{CommandKey, Resolved, ResolvedCommand, ResolvedScope, ScopeKey},
  ExecutionContext,
};

use crate::{ipc::InvokeError, sealed::ManagerBase, Runtime};

use super::{CommandArg, CommandItem};

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  #[cfg(debug_assertions)]
  acl: BTreeMap<String, crate::utils::acl::plugin::Manifest>,
  allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
  pub(crate) scope_manager: ScopeManager,
}

/// The origin trying to access the IPC.
pub enum Origin {
  /// Local app origin.
  Local,
  /// Remote origin.
  Remote {
    /// Remote origin domain.
    domain: String,
  },
}

impl Display for Origin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Local => write!(f, "local"),
      Self::Remote { domain } => write!(f, "remote: {domain}"),
    }
  }
}

impl Origin {
  fn matches(&self, context: &ExecutionContext) -> bool {
    match (self, context) {
      (Self::Local, ExecutionContext::Local) => true,
      (
        Self::Remote { domain },
        ExecutionContext::Remote {
          domain: domain_pattern,
        },
      ) => domain_pattern.matches(domain),
      _ => false,
    }
  }
}

impl RuntimeAuthority {
  pub(crate) fn new(resolved_acl: Resolved) -> Self {
    let command_cache = resolved_acl
      .command_scope
      .keys()
      .map(|key| (*key, <TypeMap![Send + Sync]>::new()))
      .collect();
    Self {
      #[cfg(debug_assertions)]
      acl: resolved_acl.acl,
      allowed_commands: resolved_acl.allowed_commands,
      denied_commands: resolved_acl.denied_commands,
      scope_manager: ScopeManager {
        command_scope: resolved_acl.command_scope,
        global_scope: resolved_acl.global_scope,
        command_cache,
        global_scope_cache: Default::default(),
      },
    }
  }

  #[cfg(debug_assertions)]
  pub(crate) fn resolve_access_message(
    &self,
    plugin: &str,
    command_name: &str,
    window: &str,
    origin: &Origin,
  ) -> String {
    fn print_references(resolved: &ResolvedCommand) -> String {
      resolved
        .referenced_by
        .iter()
        .map(|r| format!("capability: {}, permission: {}", r.capability, r.permission))
        .collect::<Vec<_>>()
        .join(" || ")
    }

    fn has_permissions_allowing_command(
      manifest: &crate::utils::acl::plugin::Manifest,
      set: &crate::utils::acl::PermissionSet,
      command: &str,
    ) -> bool {
      for permission_id in &set.permissions {
        if permission_id == "default" {
          if let Some(default) = &manifest.default_permission {
            if has_permissions_allowing_command(manifest, default, command) {
              return true;
            }
          }
        } else if let Some(ref_set) = manifest.permission_sets.get(permission_id) {
          if has_permissions_allowing_command(manifest, ref_set, command) {
            return true;
          }
        } else if let Some(permission) = manifest.permissions.get(permission_id) {
          if permission.commands.allow.contains(&command.into()) {
            return true;
          }
        }
      }
      false
    }

    let command = format!("plugin:{plugin}|{command_name}");
    if let Some((_cmd, resolved)) = self
      .denied_commands
      .iter()
      .find(|(cmd, _)| cmd.name == command && origin.matches(&cmd.context))
    {
      format!(
        "{plugin}.{command_name} denied on origin {origin}, referenced by: {}",
        print_references(resolved)
      )
    } else {
      let command_matches = self
        .allowed_commands
        .iter()
        .filter(|(cmd, _)| cmd.name == command)
        .collect::<BTreeMap<_, _>>();

      if let Some((_cmd, resolved)) = command_matches
        .iter()
        .find(|(cmd, _)| origin.matches(&cmd.context))
      {
        if resolved.windows.iter().any(|w| w.matches(window)) {
          "allowed".to_string()
        } else {
          format!("{plugin}.{command_name} not allowed on window {window}, expected one of {}, referenced by {}", resolved.windows.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", "), print_references(resolved))
        }
      } else {
        let permission_error_detail = if let Some(manifest) = self.acl.get(plugin) {
          let mut permissions_referencing_command = Vec::new();

          if let Some(default) = &manifest.default_permission {
            if has_permissions_allowing_command(manifest, default, command_name) {
              permissions_referencing_command.push("default".into());
            }
          }
          for set in manifest.permission_sets.values() {
            if has_permissions_allowing_command(manifest, set, command_name) {
              permissions_referencing_command.push(set.identifier.clone());
            }
          }
          for permission in manifest.permissions.values() {
            if permission.commands.allow.contains(&command_name.into()) {
              permissions_referencing_command.push(permission.identifier.clone());
            }
          }

          permissions_referencing_command.sort();

          format!(
            "Permissions associated with this command: {}",
            permissions_referencing_command
              .iter()
              .map(|p| format!("{plugin}:{p}"))
              .collect::<Vec<_>>()
              .join(", ")
          )
        } else {
          "Plugin did not define its manifest".to_string()
        };

        if command_matches.is_empty() {
          format!("{plugin}.{command_name} not allowed. {permission_error_detail}")
        } else {
          format!(
            "{plugin}.{command_name} not allowed on origin [{}]. Please create a capability that has this origin on the context field.\n\nFound matches for: {}\n\n{permission_error_detail}",
            origin,
            command_matches
              .iter()
              .map(|(cmd, resolved)| {
                let context = match &cmd.context {
                  ExecutionContext::Local => "[local]".to_string(),
                  ExecutionContext::Remote { domain } => format!("[remote: {}]", domain.as_str()),
                };
                format!(
                  "- context: {context}, referenced by: {}",
                  print_references(resolved)
                )
              })
              .collect::<Vec<_>>()
              .join("\n")
          )
        }
      }
    }
  }

  /// Checks if the given IPC execution is allowed and returns the [`ResolvedCommand`] if it is.
  pub fn resolve_access(
    &self,
    command: &str,
    window: &str,
    origin: &Origin,
  ) -> Option<&ResolvedCommand> {
    if self
      .denied_commands
      .keys()
      .any(|cmd| cmd.name == command && origin.matches(&cmd.context))
    {
      None
    } else {
      self
        .allowed_commands
        .iter()
        .find(|(cmd, _)| cmd.name == command && origin.matches(&cmd.context))
        .map(|(_cmd, resolved)| resolved)
        .filter(|resolved| resolved.windows.iter().any(|w| w.matches(window)))
    }
  }
}

/// List of allowed and denied objects that match either the command-specific or plugin global scope criterias.
#[derive(Debug)]
pub struct ScopeValue<T: Debug + DeserializeOwned + Send + Sync + 'static> {
  allow: Vec<T>,
  deny: Vec<T>,
}

impl<T: Debug + DeserializeOwned + Send + Sync + 'static> ScopeValue<T> {
  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<T> {
    &self.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<T> {
    &self.deny
  }
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
          .webview
          .manager()
          .runtime_authority
          .scope_manager
          .get_command_scope_typed(&scope_id)
          .unwrap_or_default()
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
    command
      .plugin
      .and_then(|plugin| {
        command
          .message
          .webview
          .manager()
          .runtime_authority
          .scope_manager
          .get_global_scope_typed(plugin)
          .ok()
      })
      .map(GlobalScope)
      .ok_or_else(|| InvokeError::from_anyhow(anyhow::anyhow!("global scope not found")))
  }
}

#[derive(Debug)]
pub struct ScopeManager {
  command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  global_scope: BTreeMap<String, ResolvedScope>,
  command_cache: BTreeMap<ScopeKey, TypeMap![Send + Sync]>,
  global_scope_cache: TypeMap![Send + Sync],
}

impl ScopeManager {
  pub(crate) fn get_global_scope_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
    plugin: &str,
  ) -> crate::Result<&ScopeValue<T>> {
    match self.global_scope_cache.try_get() {
      Some(cached) => Ok(cached),
      None => {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        if let Some(global_scope) = self.global_scope.get(plugin) {
          for allowed in &global_scope.allow {
            allow.push(serde_json::from_value(allowed.clone().into())?);
          }
          for denied in &global_scope.deny {
            deny.push(serde_json::from_value(denied.clone().into())?);
          }
        }

        let scope = ScopeValue { allow, deny };
        let _ = self.global_scope_cache.set(scope);
        Ok(self.global_scope_cache.get())
      }
    }
  }

  fn get_command_scope_typed<T: Send + Sync + DeserializeOwned + Debug + 'static>(
    &self,
    key: &ScopeKey,
  ) -> crate::Result<Option<&ScopeValue<T>>> {
    let cache = self.command_cache.get(key).unwrap();
    match cache.try_get() {
      cached @ Some(_) => Ok(cached),
      None => match self.command_scope.get(key).map(|r| {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        for allowed in &r.allow {
          allow.push(serde_json::from_value(allowed.clone().into())?);
        }
        for denied in &r.deny {
          deny.push(serde_json::from_value(denied.clone().into())?);
        }

        crate::Result::Ok(Some(ScopeValue { allow, deny }))
      }) {
        None => Ok(None),
        Some(value) => {
          let _ = cache.set(value);
          Ok(cache.try_get())
        }
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use glob::Pattern;
  use tauri_utils::acl::{
    resolved::{CommandKey, Resolved, ResolvedCommand},
    ExecutionContext,
  };

  use crate::command::Origin;

  use super::RuntimeAuthority;

  #[test]
  fn window_glob_pattern_matches() {
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Local,
    };
    let window = "main-*";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(Resolved {
      allowed_commands,
      ..Default::default()
    });

    assert_eq!(
      authority.resolve_access(
        &command.name,
        &window.replace('*', "something"),
        &Origin::Local
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_matches() {
    let domain = "tauri.app";
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Remote {
        domain: Pattern::new(domain).unwrap(),
      },
    };
    let window = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(Resolved {
      allowed_commands,
      ..Default::default()
    });

    assert_eq!(
      authority.resolve_access(
        &command.name,
        window,
        &Origin::Remote {
          domain: domain.into()
        }
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_glob_pattern_matches() {
    let domain = "tauri.*";
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Remote {
        domain: Pattern::new(domain).unwrap(),
      },
    };
    let window = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(Resolved {
      allowed_commands,
      ..Default::default()
    });

    assert_eq!(
      authority.resolve_access(
        &command.name,
        window,
        &Origin::Remote {
          domain: domain.replace('*', "studio")
        }
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn remote_context_denied() {
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Local,
    };
    let window = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(Resolved {
      allowed_commands,
      ..Default::default()
    });

    assert!(authority
      .resolve_access(
        &command.name,
        window,
        &Origin::Remote {
          domain: "tauri.app".into()
        }
      )
      .is_none());
  }

  #[test]
  fn denied_command_takes_precendence() {
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Local,
    };
    let window = "main";
    let windows = vec![Pattern::new(window).unwrap()];
    let allowed_commands = [(
      command.clone(),
      ResolvedCommand {
        windows: windows.clone(),
        ..Default::default()
      },
    )]
    .into_iter()
    .collect();
    let denied_commands = [(
      command.clone(),
      ResolvedCommand {
        windows: windows.clone(),
        ..Default::default()
      },
    )]
    .into_iter()
    .collect();

    let authority = RuntimeAuthority::new(Resolved {
      allowed_commands,
      denied_commands,
      ..Default::default()
    });

    assert!(authority
      .resolve_access(&command.name, window, &Origin::Local)
      .is_none());
  }
}
