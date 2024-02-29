// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use state::TypeMap;

use tauri_utils::acl::{
  capability::{Capability, CapabilityFile, PermissionEntry},
  manifest::Manifest,
  Value, APP_ACL_KEY,
};
use tauri_utils::acl::{
  resolved::{CommandKey, Resolved, ResolvedCommand, ResolvedScope, ScopeKey},
  ExecutionContext, Scopes,
};

use crate::{ipc::InvokeError, sealed::ManagerBase, Runtime};
use crate::{AppHandle, Manager};

use super::{CommandArg, CommandItem};

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  acl: BTreeMap<String, crate::utils::acl::manifest::Manifest>,
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
    /// Remote URL.
    url: String,
  },
}

impl Display for Origin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Local => write!(f, "local"),
      Self::Remote { url } => write!(f, "remote: {url}"),
    }
  }
}

impl Origin {
  fn matches(&self, context: &ExecutionContext) -> bool {
    match (self, context) {
      (Self::Local, ExecutionContext::Local) => true,
      (Self::Remote { url }, ExecutionContext::Remote { url: url_pattern }) => {
        url_pattern.matches(url)
      }
      _ => false,
    }
  }
}

/// A capability that can be added at runtime.
pub trait RuntimeCapability {
  /// Creates the capability file.
  fn build(self) -> CapabilityFile;
}

impl<T: AsRef<str>> RuntimeCapability for T {
  fn build(self) -> CapabilityFile {
    self.as_ref().parse().expect("invalid capability")
  }
}

/// A builder for a [`Capability`].
pub struct CapabilityBuilder(Capability);

impl CapabilityBuilder {
  /// Creates a new capability builder with a unique identifier.
  pub fn new(identifier: impl Into<String>) -> Self {
    Self(Capability {
      identifier: identifier.into(),
      description: "".into(),
      remote: None,
      local: true,
      windows: Vec::new(),
      webviews: Vec::new(),
      permissions: Vec::new(),
      platforms: Vec::new(),
    })
  }

  /// Allows this capability to be used by a remote URL.
  pub fn remote(mut self, url: String) -> Self {
    self
      .0
      .remote
      .get_or_insert_with(Default::default)
      .urls
      .push(url);
    self
  }

  /// Whether this capability is applied on local app URLs or not. Defaults to `true`.
  pub fn local(mut self, local: bool) -> Self {
    self.0.local = local;
    self
  }

  /// Link this capability to the given window label.
  pub fn window(mut self, window: impl Into<String>) -> Self {
    self.0.windows.push(window.into());
    self
  }

  /// Link this capability to the a list of window labels.
  pub fn windows(mut self, windows: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self.0.windows.extend(windows.into_iter().map(|w| w.into()));
    self
  }

  /// Link this capability to the given webview label.
  pub fn webview(mut self, webview: impl Into<String>) -> Self {
    self.0.webviews.push(webview.into());
    self
  }

  /// Link this capability to the a list of window labels.
  pub fn webviews(mut self, webviews: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self
      .0
      .webviews
      .extend(webviews.into_iter().map(|w| w.into()));
    self
  }

  /// Add a new permission to this capability.
  pub fn permission(mut self, permission: impl Into<String>) -> Self {
    let permission = permission.into();
    self.0.permissions.push(PermissionEntry::PermissionRef(
      permission
        .clone()
        .try_into()
        .unwrap_or_else(|_| panic!("invalid permission identifier '{permission}'")),
    ));
    self
  }

  /// Add a new scoped permission to this capability.
  pub fn permission_scoped<T: Serialize>(
    mut self,
    permission: impl Into<String>,
    allowed: Vec<T>,
    denied: Vec<T>,
  ) -> Self {
    let permission = permission.into();
    let identifier = permission
      .clone()
      .try_into()
      .unwrap_or_else(|_| panic!("invalid permission identifier '{permission}'"));

    self
      .0
      .permissions
      .push(PermissionEntry::ExtendedPermission {
        identifier,
        scope: Scopes {
          allow: Some(
            allowed
              .into_iter()
              .map(|a| {
                serde_json::to_value(a)
                  .expect("failed to serialize scope")
                  .into()
              })
              .collect(),
          ),
          deny: Some(
            denied
              .into_iter()
              .map(|a| {
                serde_json::to_value(a)
                  .expect("failed to serialize scope")
                  .into()
              })
              .collect(),
          ),
        },
      });
    self
  }
}

impl RuntimeCapability for CapabilityBuilder {
  fn build(self) -> CapabilityFile {
    CapabilityFile::Capability(self.0)
  }
}

impl RuntimeAuthority {
  #[doc(hidden)]
  pub fn new(acl: BTreeMap<String, Manifest>, resolved_acl: Resolved) -> Self {
    let command_cache = resolved_acl
      .command_scope
      .keys()
      .map(|key| (*key, <TypeMap![Send + Sync]>::new()))
      .collect();
    Self {
      acl,
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

  pub(crate) fn has_app_manifest(&self) -> bool {
    self.acl.contains_key(APP_ACL_KEY)
  }

  #[doc(hidden)]
  pub fn __allow_command(&mut self, command: String, context: ExecutionContext) {
    self.allowed_commands.insert(
      CommandKey {
        name: command,
        context,
      },
      ResolvedCommand {
        windows: vec!["*".parse().unwrap()],
        ..Default::default()
      },
    );
  }

  /// Adds the given capability to the runtime authority.
  pub fn add_capability(&mut self, capability: impl RuntimeCapability) -> crate::Result<()> {
    let mut capabilities = BTreeMap::new();
    match capability.build() {
      CapabilityFile::Capability(c) => {
        capabilities.insert(c.identifier.clone(), c);
      }
      CapabilityFile::List {
        capabilities: capabilities_list,
      } => {
        capabilities.extend(
          capabilities_list
            .into_iter()
            .map(|c| (c.identifier.clone(), c)),
        );
      }
    }

    let resolved = Resolved::resolve(
      &self.acl,
      capabilities,
      tauri_utils::platform::Target::current(),
    )
    .unwrap();

    // fill global scope
    for (plugin, global_scope) in resolved.global_scope {
      let global_scope_entry = self.scope_manager.global_scope.entry(plugin).or_default();

      global_scope_entry.allow.extend(global_scope.allow);
      global_scope_entry.deny.extend(global_scope.deny);

      self.scope_manager.global_scope_cache = Default::default();
    }

    // denied commands
    for (cmd_key, resolved_cmd) in resolved.denied_commands {
      let entry = self.denied_commands.entry(cmd_key).or_default();

      entry.windows.extend(resolved_cmd.windows);
      #[cfg(debug_assertions)]
      entry.referenced_by.extend(resolved_cmd.referenced_by);
    }

    // allowed commands
    for (cmd_key, resolved_cmd) in resolved.allowed_commands {
      let entry = self.allowed_commands.entry(cmd_key).or_default();

      entry.windows.extend(resolved_cmd.windows);
      #[cfg(debug_assertions)]
      entry.referenced_by.extend(resolved_cmd.referenced_by);

      // fill command scope
      if let Some(scope_id) = resolved_cmd.scope {
        let command_scope = resolved.command_scope.get(&scope_id).unwrap();

        let command_scope_entry = self
          .scope_manager
          .command_scope
          .entry(scope_id)
          .or_default();
        command_scope_entry
          .allow
          .extend(command_scope.allow.clone());
        command_scope_entry.deny.extend(command_scope.deny.clone());

        self.scope_manager.command_cache.remove(&scope_id);
      }
    }

    Ok(())
  }

  #[cfg(debug_assertions)]
  pub(crate) fn resolve_access_message(
    &self,
    key: &str,
    command_name: &str,
    window: &str,
    webview: &str,
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
      manifest: &crate::utils::acl::manifest::Manifest,
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

    let command = if key == APP_ACL_KEY {
      command_name.to_string()
    } else {
      format!("plugin:{key}|{command_name}")
    };

    let command_pretty_name = if key == APP_ACL_KEY {
      command_name.to_string()
    } else {
      format!("{key}.{command_name}")
    };

    if let Some((_cmd, resolved)) = self
      .denied_commands
      .iter()
      .find(|(cmd, _)| cmd.name == command && origin.matches(&cmd.context))
    {
      format!(
        "{command_pretty_name} denied on origin {origin}, referenced by: {}",
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
        if resolved.webviews.iter().any(|w| w.matches(webview))
          || resolved.windows.iter().any(|w| w.matches(window))
        {
          "allowed".to_string()
        } else {
          format!("{command_pretty_name} not allowed on window {window}, webview {webview}, allowed windows: {}, allowed webviews: {}, referenced by {}",
            resolved.windows.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", "),
            resolved.webviews.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", "),
            print_references(resolved)
          )
        }
      } else {
        let permission_error_detail = if let Some(manifest) = self.acl.get(key) {
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
              .map(|p| if key == APP_ACL_KEY {
                p.to_string()
              } else {
                format!("{key}:{p}")
              })
              .collect::<Vec<_>>()
              .join(", ")
          )
        } else {
          "Plugin did not define its manifest".to_string()
        };

        if command_matches.is_empty() {
          format!("{command_pretty_name} not allowed. {permission_error_detail}")
        } else {
          format!(
            "{command_pretty_name} not allowed on origin [{}]. Please create a capability that has this origin on the context field.\n\nFound matches for: {}\n\n{permission_error_detail}",
            origin,
            command_matches
              .iter()
              .map(|(cmd, resolved)| {
                let context = match &cmd.context {
                  ExecutionContext::Local => "[local]".to_string(),
                  ExecutionContext::Remote { url } => format!("[remote: {}]", url.as_str()),
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
    webview: &str,
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
        .filter(|resolved| {
          resolved.webviews.iter().any(|w| w.matches(webview))
            || resolved.windows.iter().any(|w| w.matches(window))
        })
    }
  }
}

/// List of allowed and denied objects that match either the command-specific or plugin global scope criterias.
#[derive(Debug)]
pub struct ScopeValue<T: ScopeObject> {
  allow: Arc<Vec<T>>,
  deny: Arc<Vec<T>>,
}

impl<T: ScopeObject> ScopeValue<T> {
  fn clone(&self) -> Self {
    Self {
      allow: self.allow.clone(),
      deny: self.deny.clone(),
    }
  }

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
pub struct CommandScope<T: ScopeObject>(ScopeValue<T>);

impl<T: ScopeObject> CommandScope<T> {
  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<T> {
    &self.0.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<T> {
    &self.0.deny
  }
}

impl<'a, R: Runtime, T: ScopeObject> CommandArg<'a, R> for CommandScope<T> {
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`CommandScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    if let Some(scope_id) = command.acl.as_ref().and_then(|resolved| resolved.scope) {
      Ok(CommandScope(
        command
          .message
          .webview
          .manager()
          .runtime_authority
          .lock()
          .unwrap()
          .scope_manager
          .get_command_scope_typed(command.message.webview.app_handle(), &scope_id)?,
      ))
    } else {
      Ok(CommandScope(ScopeValue {
        allow: Default::default(),
        deny: Default::default(),
      }))
    }
  }
}

/// Global access scope that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct GlobalScope<T: ScopeObject>(ScopeValue<T>);

impl<T: ScopeObject> GlobalScope<T> {
  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<T> {
    &self.0.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<T> {
    &self.0.deny
  }
}

impl<'a, R: Runtime, T: ScopeObject> CommandArg<'a, R> for GlobalScope<T> {
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`GlobalScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    command
      .message
      .webview
      .manager()
      .runtime_authority
      .lock()
      .unwrap()
      .scope_manager
      .get_global_scope_typed(
        command.message.webview.app_handle(),
        command.plugin.unwrap_or(APP_ACL_KEY),
      )
      .map_err(InvokeError::from_error)
      .map(GlobalScope)
  }
}

#[derive(Debug)]
pub struct ScopeManager {
  command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  global_scope: BTreeMap<String, ResolvedScope>,
  command_cache: BTreeMap<ScopeKey, TypeMap![Send + Sync]>,
  global_scope_cache: TypeMap![Send + Sync],
}

/// Marks a type as a scope object.
///
/// Usually you will just rely on [`serde::de::DeserializeOwned`] instead of implementing it manually,
/// though this is useful if you need to do some initialization logic on the type itself.
pub trait ScopeObject: Sized + Send + Sync + Debug + 'static {
  /// The error type.
  type Error: std::error::Error + Send + Sync;
  /// Deserialize the raw scope value.
  fn deserialize<R: Runtime>(app: &AppHandle<R>, raw: Value) -> Result<Self, Self::Error>;
}

impl<T: Send + Sync + Debug + DeserializeOwned + 'static> ScopeObject for T {
  type Error = serde_json::Error;
  fn deserialize<R: Runtime>(_app: &AppHandle<R>, raw: Value) -> Result<Self, Self::Error> {
    serde_json::from_value(raw.into())
  }
}

impl ScopeManager {
  pub(crate) fn get_global_scope_typed<R: Runtime, T: ScopeObject>(
    &self,
    app: &AppHandle<R>,
    key: &str,
  ) -> crate::Result<ScopeValue<T>> {
    match self.global_scope_cache.try_get::<ScopeValue<T>>() {
      Some(cached) => Ok(cached.clone()),
      None => {
        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        if let Some(global_scope) = self.global_scope.get(key) {
          for allowed in &global_scope.allow {
            allow.push(
              T::deserialize(app, allowed.clone())
                .map_err(|e| crate::Error::CannotDeserializeScope(Box::new(e)))?,
            );
          }
          for denied in &global_scope.deny {
            deny.push(
              T::deserialize(app, denied.clone())
                .map_err(|e| crate::Error::CannotDeserializeScope(Box::new(e)))?,
            );
          }
        }

        let scope = ScopeValue {
          allow: Arc::new(allow),
          deny: Arc::new(deny),
        };
        self.global_scope_cache.set(scope.clone());
        Ok(scope)
      }
    }
  }

  fn get_command_scope_typed<R: Runtime, T: ScopeObject>(
    &self,
    app: &AppHandle<R>,
    key: &ScopeKey,
  ) -> crate::Result<ScopeValue<T>> {
    let cache = self.command_cache.get(key).unwrap();
    match cache.try_get::<ScopeValue<T>>() {
      Some(cached) => Ok(cached.clone()),
      None => {
        let resolved_scope = self
          .command_scope
          .get(key)
          .unwrap_or_else(|| panic!("missing command scope for key {key}"));

        let mut allow: Vec<T> = Vec::new();
        let mut deny: Vec<T> = Vec::new();

        for allowed in &resolved_scope.allow {
          allow.push(
            T::deserialize(app, allowed.clone())
              .map_err(|e| crate::Error::CannotDeserializeScope(Box::new(e)))?,
          );
        }
        for denied in &resolved_scope.deny {
          deny.push(
            T::deserialize(app, denied.clone())
              .map_err(|e| crate::Error::CannotDeserializeScope(Box::new(e)))?,
          );
        }

        let value = ScopeValue {
          allow: Arc::new(allow),
          deny: Arc::new(deny),
        };

        let _ = cache.set(value.clone());
        Ok(value)
      }
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

  use crate::ipc::Origin;

  use super::RuntimeAuthority;

  #[test]
  fn window_glob_pattern_matches() {
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Local,
    };
    let window = "main-*";
    let webview = "other-*";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert_eq!(
      authority.resolve_access(
        &command.name,
        &window.replace('*', "something"),
        webview,
        &Origin::Local
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn webview_glob_pattern_matches() {
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Local,
    };
    let window = "other-*";
    let webview = "main-*";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      webviews: vec![Pattern::new(webview).unwrap()],
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert_eq!(
      authority.resolve_access(
        &command.name,
        window,
        &webview.replace('*', "something"),
        &Origin::Local
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_matches() {
    let url = "https://tauri.app";
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Remote {
        url: Pattern::new(url).unwrap(),
      },
    };
    let window = "main";
    let webview = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert_eq!(
      authority.resolve_access(
        &command.name,
        window,
        webview,
        &Origin::Remote { url: url.into() }
      ),
      Some(&resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_glob_pattern_matches() {
    let url = "http://tauri.*";
    let command = CommandKey {
      name: "my-command".into(),
      context: ExecutionContext::Remote {
        url: Pattern::new(url).unwrap(),
      },
    };
    let window = "main";
    let webview = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert_eq!(
      authority.resolve_access(
        &command.name,
        window,
        webview,
        &Origin::Remote {
          url: url.replace('*', "studio")
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
    let webview = "main";

    let resolved_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      scope: None,
      ..Default::default()
    };
    let allowed_commands = [(command.clone(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert!(authority
      .resolve_access(
        &command.name,
        window,
        webview,
        &Origin::Remote {
          url: "https://tauri.app".into()
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
    let webview = "main";
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

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        denied_commands,
        ..Default::default()
      },
    );

    assert!(authority
      .resolve_access(&command.name, window, webview, &Origin::Local)
      .is_none());
  }
}
