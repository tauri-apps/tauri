// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tauri_utils::acl::{
  capability::{Capability, CapabilityFile, PermissionEntry},
  manifest::Manifest,
  Value, APP_ACL_KEY,
};
use tauri_utils::acl::{
  resolved::{Resolved, ResolvedCommand, ResolvedScope, ScopeKey},
  ExecutionContext, Scopes,
};
use tauri_utils::platform::Target;

use url::Url;

use crate::{ipc::InvokeError, sealed::ManagerBase, Runtime};
use crate::{AppHandle, Manager, StateManager, Webview};

use super::{CommandArg, CommandItem};

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  acl: BTreeMap<String, crate::utils::acl::manifest::Manifest>,
  allowed_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  denied_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  pub(crate) scope_manager: ScopeManager,
}

/// The origin trying to access the IPC.
pub enum Origin {
  /// Local app origin.
  Local,
  /// Remote origin.
  Remote {
    /// Remote URL.
    url: Url,
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
        url_pattern.test(url)
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
      platforms: None,
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

    let allowed_scope = allowed
      .into_iter()
      .map(|a| {
        serde_json::to_value(a)
          .expect("failed to serialize scope")
          .into()
      })
      .collect();
    let denied_scope = denied
      .into_iter()
      .map(|a| {
        serde_json::to_value(a)
          .expect("failed to serialize scope")
          .into()
      })
      .collect();
    let scope = Scopes {
      allow: Some(allowed_scope),
      deny: Some(denied_scope),
    };

    self
      .0
      .permissions
      .push(PermissionEntry::ExtendedPermission { identifier, scope });
    self
  }

  /// Adds a target platform for this capability.
  ///
  /// By default all platforms are applied.
  pub fn platform(mut self, platform: Target) -> Self {
    self
      .0
      .platforms
      .get_or_insert_with(Default::default)
      .push(platform);
    self
  }

  /// Adds target platforms for this capability.
  ///
  /// By default all platforms are applied.
  pub fn platforms(mut self, platforms: impl IntoIterator<Item = Target>) -> Self {
    self
      .0
      .platforms
      .get_or_insert_with(Default::default)
      .extend(platforms);
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
      .map(|key| (*key, StateManager::new()))
      .collect();
    Self {
      acl,
      allowed_commands: resolved_acl.allowed_commands,
      denied_commands: resolved_acl.denied_commands,
      scope_manager: ScopeManager {
        command_scope: resolved_acl.command_scope,
        global_scope: resolved_acl.global_scope,
        command_cache,
        global_scope_cache: StateManager::new(),
      },
    }
  }

  pub(crate) fn has_app_manifest(&self) -> bool {
    self.acl.contains_key(APP_ACL_KEY)
  }

  #[doc(hidden)]
  pub fn __allow_command(&mut self, command: String, context: ExecutionContext) {
    self.allowed_commands.insert(
      command,
      vec![ResolvedCommand {
        context,
        windows: vec!["*".parse().unwrap()],
        ..Default::default()
      }],
    );
  }

  /// Adds the given capability to the runtime authority.
  pub fn add_capability(&mut self, capability: impl RuntimeCapability) -> crate::Result<()> {
    let mut capabilities = BTreeMap::new();
    match capability.build() {
      CapabilityFile::Capability(c) => {
        capabilities.insert(c.identifier.clone(), c);
      }

      CapabilityFile::List(capabilities_list)
      | CapabilityFile::NamedList {
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

      self.scope_manager.global_scope_cache = StateManager::new();
    }

    // denied commands
    for (cmd_key, resolved_cmds) in resolved.denied_commands {
      let entry = self.denied_commands.entry(cmd_key).or_default();
      entry.extend(resolved_cmds);
    }

    // allowed commands
    for (cmd_key, resolved_cmds) in resolved.allowed_commands {
      // fill command scope
      for resolved_cmd in &resolved_cmds {
        if let Some(scope_id) = resolved_cmd.scope_id {
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

          self
            .scope_manager
            .command_cache
            .insert(scope_id, StateManager::new());
        }
      }

      let entry = self.allowed_commands.entry(cmd_key).or_default();
      entry.extend(resolved_cmds);
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
    fn print_references(resolved: &[ResolvedCommand]) -> String {
      resolved
        .iter()
        .map(|r| {
          format!(
            "capability: {}, permission: {}",
            r.referenced_by.capability, r.referenced_by.permission
          )
        })
        .collect::<Vec<_>>()
        .join(" || ")
    }

    fn print_allowed_on(resolved: &[ResolvedCommand]) -> String {
      if resolved.is_empty() {
        "command not allowed on any window/webview/URL context".to_string()
      } else {
        let mut s = "allowed on: ".to_string();

        let last_index = resolved.len() - 1;
        for (index, cmd) in resolved.iter().enumerate() {
          let windows = cmd
            .windows
            .iter()
            .map(|w| format!("\"{}\"", w.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
          let webviews = cmd
            .webviews
            .iter()
            .map(|w| format!("\"{}\"", w.as_str()))
            .collect::<Vec<_>>()
            .join(", ");

          s.push('[');

          if !windows.is_empty() {
            s.push_str(&format!("windows: {windows}, "));
          }

          if !webviews.is_empty() {
            s.push_str(&format!("webviews: {webviews}, "));
          }

          match &cmd.context {
            ExecutionContext::Local => s.push_str("URL: local"),
            ExecutionContext::Remote { url } => s.push_str(&format!("URL: {}", url.as_str())),
          }

          s.push(']');

          if index != last_index {
            s.push_str(", ");
          }
        }

        s
      }
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

    if let Some(resolved) = self.denied_commands.get(&command) {
      format!(
        "{command_pretty_name} explicitly denied on origin {origin}\n\nreferenced by: {}",
        print_references(resolved)
      )
    } else {
      let command_matches = self.allowed_commands.get(&command);

      if let Some(resolved) = self.allowed_commands.get(&command) {
        let resolved_matching_origin = resolved
          .iter()
          .filter(|cmd| origin.matches(&cmd.context))
          .collect::<Vec<&ResolvedCommand>>();
        if resolved_matching_origin
          .iter()
          .any(|cmd| cmd.webviews.iter().any(|w| w.matches(webview)))
          || resolved_matching_origin
            .iter()
            .any(|cmd| cmd.windows.iter().any(|w| w.matches(window)))
        {
          "allowed".to_string()
        } else {
          format!("{command_pretty_name} not allowed on window \"{window}\", webview \"{webview}\", URL: {}\n\n{}\n\nreferenced by: {}",
            match origin {
              Origin::Local => "local",
              Origin::Remote { url } => url.as_str()
            },
            print_allowed_on(resolved),
            print_references(resolved)
          )
        }
      } else {
        let permission_error_detail = if let Some(manifest) = self
          .acl
          .get(key)
          .or_else(|| self.acl.get(&format!("core:{key}")))
        {
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

          let associated_permissions = permissions_referencing_command
            .iter()
            .map(|p| {
              if key == APP_ACL_KEY {
                p.to_string()
              } else {
                format!("{key}:{p}")
              }
            })
            .collect::<Vec<_>>()
            .join(", ");

          if associated_permissions.is_empty() {
            "Command not found".to_string()
          } else {
            format!("Permissions associated with this command: {associated_permissions}")
          }
        } else {
          "Plugin not found".to_string()
        };

        if let Some(resolved_cmds) = command_matches {
          format!(
            "{command_pretty_name} not allowed on origin [{}]. Please create a capability that has this origin on the context field.\n\nFound matches for: {}\n\n{permission_error_detail}",
            origin,
            resolved_cmds
              .iter()
              .map(|resolved| {
                let context = match &resolved.context {
                  ExecutionContext::Local => "[local]".to_string(),
                  ExecutionContext::Remote { url } => format!("[remote: {}]", url.as_str()),
                };
                format!(
                  "- context: {context}, referenced by: capability: {}, permission: {}",
                  resolved.referenced_by.capability,
                  resolved.referenced_by.permission
                )
              })
              .collect::<Vec<_>>()
              .join("\n")
          )
        } else {
          format!("{command_pretty_name} not allowed. {permission_error_detail}")
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
  ) -> Option<Vec<ResolvedCommand>> {
    if self
      .denied_commands
      .get(command)
      .map(|resolved| resolved.iter().any(|cmd| origin.matches(&cmd.context)))
      .is_some()
    {
      None
    } else {
      self.allowed_commands.get(command).and_then(|resolved| {
        let resolved_cmds = resolved
          .iter()
          .filter(|cmd| {
            origin.matches(&cmd.context)
              && (cmd.webviews.iter().any(|w| w.matches(webview))
                || cmd.windows.iter().any(|w| w.matches(window)))
          })
          .cloned()
          .collect::<Vec<_>>();
        if resolved_cmds.is_empty() {
          None
        } else {
          Some(resolved_cmds)
        }
      })
    }
  }
}

/// List of allowed and denied objects that match either the command-specific or plugin global scope criteria.
#[derive(Debug)]
pub struct ScopeValue<T: ScopeObject> {
  allow: Arc<Vec<Arc<T>>>,
  deny: Arc<Vec<Arc<T>>>,
}

impl<T: ScopeObject> ScopeValue<T> {
  fn clone(&self) -> Self {
    Self {
      allow: self.allow.clone(),
      deny: self.deny.clone(),
    }
  }

  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<Arc<T>> {
    &self.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<Arc<T>> {
    &self.deny
  }
}

/// Access scope for a command that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct CommandScope<T: ScopeObject> {
  allow: Vec<Arc<T>>,
  deny: Vec<Arc<T>>,
}

impl<T: ScopeObject> CommandScope<T> {
  pub(crate) fn resolve<R: Runtime>(
    webview: &Webview<R>,
    scope_ids: Vec<u64>,
  ) -> crate::Result<Self> {
    let mut allow = Vec::new();
    let mut deny = Vec::new();

    for scope_id in scope_ids {
      let scope = webview
        .manager()
        .runtime_authority
        .lock()
        .unwrap()
        .scope_manager
        .get_command_scope_typed::<R, T>(webview.app_handle(), &scope_id)?;

      for s in scope.allows() {
        allow.push(s.clone());
      }
      for s in scope.denies() {
        deny.push(s.clone());
      }
    }

    Ok(CommandScope { allow, deny })
  }

  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<Arc<T>> {
    &self.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<Arc<T>> {
    &self.deny
  }
}

impl<T: ScopeObjectMatch> CommandScope<T> {
  /// Ensure all deny scopes were not matched and any allow scopes were.
  ///
  /// This **WILL** return `true` if the allow scopes are empty and the deny
  /// scopes did not trigger. If you require at least one allow scope, then
  /// ensure the allow scopes are not empty before calling this method.
  ///
  /// ```
  /// # use tauri::ipc::CommandScope;
  /// # fn command(scope: CommandScope<()>) -> Result<(), &'static str> {
  /// if scope.allows().is_empty() {
  ///   return Err("you need to specify at least 1 allow scope!");
  /// }
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Example
  ///
  /// ```
  /// # use serde::{Serialize, Deserialize};
  /// # use url::Url;
  /// # use tauri::{ipc::{CommandScope, ScopeObjectMatch}, command};
  /// #
  /// #[derive(Debug, Clone, Serialize, Deserialize)]
  /// # pub struct Scope;
  /// #
  /// # impl ScopeObjectMatch for Scope {
  /// #   type Input = str;
  /// #
  /// #   fn matches(&self, input: &str) -> bool {
  /// #     true
  /// #   }
  /// # }
  /// #
  /// # fn do_work(_: String) -> Result<String, &'static str> {
  /// #   Ok("Output".into())
  /// # }
  /// #
  /// #[command]
  /// fn my_command(scope: CommandScope<Scope>, input: String) -> Result<String, &'static str> {
  ///   if scope.matches(&input) {
  ///     do_work(input)
  ///   } else {
  ///     Err("Scope didn't match input")
  ///   }
  /// }
  /// ```
  pub fn matches(&self, input: &T::Input) -> bool {
    // first make sure the input doesn't match any existing deny scope
    if self.deny.iter().any(|s| s.matches(input)) {
      return false;
    }

    // if there are allow scopes, ensure the input matches at least 1
    if self.allow.is_empty() {
      true
    } else {
      self.allow.iter().any(|s| s.matches(input))
    }
  }
}

impl<'a, R: Runtime, T: ScopeObject> CommandArg<'a, R> for CommandScope<T> {
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`CommandScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    let scope_ids = command.acl.as_ref().map(|resolved| {
      resolved
        .iter()
        .filter_map(|cmd| cmd.scope_id)
        .collect::<Vec<_>>()
    });
    if let Some(scope_ids) = scope_ids {
      CommandScope::resolve(&command.message.webview, scope_ids).map_err(Into::into)
    } else {
      Ok(CommandScope {
        allow: Default::default(),
        deny: Default::default(),
      })
    }
  }
}

/// Global access scope that can be retrieved directly in the command function.
#[derive(Debug)]
pub struct GlobalScope<T: ScopeObject>(ScopeValue<T>);

impl<T: ScopeObject> GlobalScope<T> {
  pub(crate) fn resolve<R: Runtime>(webview: &Webview<R>, plugin: &str) -> crate::Result<Self> {
    webview
      .manager()
      .runtime_authority
      .lock()
      .unwrap()
      .scope_manager
      .get_global_scope_typed(webview.app_handle(), plugin)
      .map(Self)
  }

  /// What this access scope allows.
  pub fn allows(&self) -> &Vec<Arc<T>> {
    &self.0.allow
  }

  /// What this access scope denies.
  pub fn denies(&self) -> &Vec<Arc<T>> {
    &self.0.deny
  }
}

impl<'a, R: Runtime, T: ScopeObject> CommandArg<'a, R> for GlobalScope<T> {
  /// Grabs the [`ResolvedScope`] from the [`CommandItem`] and returns the associated [`GlobalScope`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    GlobalScope::resolve(
      &command.message.webview,
      command.plugin.unwrap_or(APP_ACL_KEY),
    )
    .map_err(InvokeError::from_error)
  }
}

#[derive(Debug)]
pub struct ScopeManager {
  command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  global_scope: BTreeMap<String, ResolvedScope>,
  command_cache: BTreeMap<ScopeKey, StateManager>,
  global_scope_cache: StateManager,
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

/// A [`ScopeObject`] whose validation can be represented as a `bool`.
///
/// # Example
///
/// ```
/// # use serde::{Deserialize, Serialize};
/// # use tauri::{ipc::ScopeObjectMatch, Url};
/// #
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// #[serde(rename_all = "camelCase")]
/// pub enum Scope {
///   Domain(Url),
///   StartsWith(String),
/// }
///
/// impl ScopeObjectMatch for Scope {
///   type Input = str;
///
///   fn matches(&self, input: &str) -> bool {
///     match self {
///       Scope::Domain(url) => {
///         let parsed: Url = match input.parse() {
///           Ok(parsed) => parsed,
///           Err(_) => return false,
///         };
///
///         let domain = parsed.domain();
///
///         domain.is_some() && domain == url.domain()
///       }
///       Scope::StartsWith(start) => input.starts_with(start),
///     }
///   }
/// }
/// ```
pub trait ScopeObjectMatch: ScopeObject {
  /// The type of input expected to validate against the scope.
  ///
  /// This will be borrowed, so if you want to match on a `&str` this type should be `str`.
  type Input: ?Sized;

  /// Check if the input matches against the scope.
  fn matches(&self, input: &Self::Input) -> bool;
}

impl ScopeManager {
  pub(crate) fn get_global_scope_typed<R: Runtime, T: ScopeObject>(
    &self,
    app: &AppHandle<R>,
    key: &str,
  ) -> crate::Result<ScopeValue<T>> {
    match self.global_scope_cache.try_get::<ScopeValue<T>>() {
      Some(cached) => Ok(cached.inner().clone()),
      None => {
        let mut allow = Vec::new();
        let mut deny = Vec::new();

        if let Some(global_scope) = self.global_scope.get(key) {
          for allowed in &global_scope.allow {
            allow
              .push(Arc::new(T::deserialize(app, allowed.clone()).map_err(
                |e| crate::Error::CannotDeserializeScope(Box::new(e)),
              )?));
          }
          for denied in &global_scope.deny {
            deny
              .push(Arc::new(T::deserialize(app, denied.clone()).map_err(
                |e| crate::Error::CannotDeserializeScope(Box::new(e)),
              )?));
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
      Some(cached) => Ok(cached.inner().clone()),
      None => {
        let resolved_scope = self
          .command_scope
          .get(key)
          .unwrap_or_else(|| panic!("missing command scope for key {key}"));

        let mut allow = Vec::new();
        let mut deny = Vec::new();

        for allowed in &resolved_scope.allow {
          allow
            .push(Arc::new(T::deserialize(app, allowed.clone()).map_err(
              |e| crate::Error::CannotDeserializeScope(Box::new(e)),
            )?));
        }
        for denied in &resolved_scope.deny {
          deny
            .push(Arc::new(T::deserialize(app, denied.clone()).map_err(
              |e| crate::Error::CannotDeserializeScope(Box::new(e)),
            )?));
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
    resolved::{Resolved, ResolvedCommand},
    ExecutionContext,
  };

  use crate::ipc::Origin;

  use super::RuntimeAuthority;

  #[test]
  fn window_glob_pattern_matches() {
    let command = "my-command";
    let window = "main-*";
    let webview = "other-*";

    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      ..Default::default()
    }];
    let allowed_commands = [(command.to_string(), resolved_cmd.clone())]
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
        command,
        &window.replace('*', "something"),
        webview,
        &Origin::Local
      ),
      Some(resolved_cmd)
    );
  }

  #[test]
  fn webview_glob_pattern_matches() {
    let command = "my-command";
    let window = "other-*";
    let webview = "main-*";

    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      webviews: vec![Pattern::new(webview).unwrap()],
      ..Default::default()
    }];
    let allowed_commands = [(command.to_string(), resolved_cmd.clone())]
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
        command,
        window,
        &webview.replace('*', "something"),
        &Origin::Local
      ),
      Some(resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_matches() {
    let url = "https://tauri.app";
    let command = "my-command";
    let window = "main";
    let webview = "main";

    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      context: ExecutionContext::Remote {
        url: url.parse().unwrap(),
      },
      ..Default::default()
    }];
    let allowed_commands = [(command.to_string(), resolved_cmd.clone())]
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
        command,
        window,
        webview,
        &Origin::Remote {
          url: url.parse().unwrap()
        }
      ),
      Some(resolved_cmd)
    );
  }

  #[test]
  fn remote_domain_glob_pattern_matches() {
    let url = "http://tauri.*";
    let command = "my-command";
    let window = "main";
    let webview = "main";

    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      context: ExecutionContext::Remote {
        url: url.parse().unwrap(),
      },
      ..Default::default()
    }];
    let allowed_commands = [(command.to_string(), resolved_cmd.clone())]
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
        command,
        window,
        webview,
        &Origin::Remote {
          url: url.replace('*', "studio").parse().unwrap()
        }
      ),
      Some(resolved_cmd)
    );
  }

  #[test]
  fn remote_context_denied() {
    let command = "my-command";
    let window = "main";
    let webview = "main";

    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      ..Default::default()
    }];
    let allowed_commands = [(command.to_string(), resolved_cmd)].into_iter().collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    assert!(authority
      .resolve_access(
        command,
        window,
        webview,
        &Origin::Remote {
          url: "https://tauri.app".parse().unwrap()
        }
      )
      .is_none());
  }

  #[test]
  fn denied_command_takes_precendence() {
    let command = "my-command";
    let window = "main";
    let webview = "main";
    let windows = vec![Pattern::new(window).unwrap()];
    let allowed_commands = [(
      command.to_string(),
      vec![ResolvedCommand {
        windows: windows.clone(),
        ..Default::default()
      }],
    )]
    .into_iter()
    .collect();
    let denied_commands = [(
      command.to_string(),
      vec![ResolvedCommand {
        windows: windows.clone(),
        ..Default::default()
      }],
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
      .resolve_access(command, window, webview, &Origin::Local)
      .is_none());
  }

  #[cfg(debug_assertions)]
  #[test]
  fn resolve_access_message() {
    use tauri_utils::acl::manifest::Manifest;

    let plugin_name = "myplugin";
    let command_allowed_on_window = "my-command-window";
    let command_allowed_on_webview_window = "my-command-webview-window";
    let window = "main-*";
    let webview = "webview-*";
    let remote_url = "http://localhost:8080";

    let referenced_by = tauri_utils::acl::resolved::ResolvedCommandReference {
      capability: "maincap".to_string(),
      permission: "allow-command".to_string(),
    };

    let resolved_window_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      referenced_by: referenced_by.clone(),
      ..Default::default()
    };
    let resolved_webview_window_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      webviews: vec![Pattern::new(webview).unwrap()],
      referenced_by: referenced_by.clone(),
      ..Default::default()
    };
    let resolved_webview_window_remote_cmd = ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      webviews: vec![Pattern::new(webview).unwrap()],
      referenced_by: referenced_by.clone(),
      context: ExecutionContext::Remote {
        url: remote_url.parse().unwrap(),
      },
      ..Default::default()
    };

    let allowed_commands = [
      (
        format!("plugin:{plugin_name}|{command_allowed_on_window}"),
        vec![resolved_window_cmd],
      ),
      (
        format!("plugin:{plugin_name}|{command_allowed_on_webview_window}"),
        vec![
          resolved_webview_window_cmd,
          resolved_webview_window_remote_cmd,
        ],
      ),
    ]
    .into_iter()
    .collect();

    let authority = RuntimeAuthority::new(
      [(
        plugin_name.to_string(),
        Manifest {
          default_permission: None,
          permissions: Default::default(),
          permission_sets: Default::default(),
          global_scope_schema: None,
        },
      )]
      .into_iter()
      .collect(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    // unknown plugin
    assert_eq!(
      authority.resolve_access_message(
        "unknown-plugin",
        command_allowed_on_window,
        window,
        webview,
        &Origin::Local
      ),
      "unknown-plugin.my-command-window not allowed. Plugin not found"
    );

    // unknown command
    assert_eq!(
      authority.resolve_access_message(
        plugin_name,
        "unknown-command",
        window,
        webview,
        &Origin::Local
      ),
      "myplugin.unknown-command not allowed. Command not found"
    );

    // window/webview do not match
    assert_eq!(
      authority.resolve_access_message(
        plugin_name,
        command_allowed_on_window,
        "other-window",
        "any-webview",
        &Origin::Local
      ),
      "myplugin.my-command-window not allowed on window \"other-window\", webview \"any-webview\", URL: local\n\nallowed on: [windows: \"main-*\", URL: local]\n\nreferenced by: capability: maincap, permission: allow-command"
    );

    // window matches, but not origin
    assert_eq!(
      authority.resolve_access_message(
        plugin_name,
        command_allowed_on_window,
        window,
        "any-webview",
        &Origin::Remote {
          url: "http://localhst".parse().unwrap()
        }
      ),
      "myplugin.my-command-window not allowed on window \"main-*\", webview \"any-webview\", URL: http://localhst/\n\nallowed on: [windows: \"main-*\", URL: local]\n\nreferenced by: capability: maincap, permission: allow-command"
    );

    // window/webview do not match
    assert_eq!(
      authority.resolve_access_message(
        plugin_name,
        command_allowed_on_webview_window,
        "other-window",
        "other-webview",
        &Origin::Local
      ),
      "myplugin.my-command-webview-window not allowed on window \"other-window\", webview \"other-webview\", URL: local\n\nallowed on: [windows: \"main-*\", webviews: \"webview-*\", URL: local], [windows: \"main-*\", webviews: \"webview-*\", URL: http://localhost:8080]\n\nreferenced by: capability: maincap, permission: allow-command || capability: maincap, permission: allow-command"
    );

    // window/webview matches, but not origin
    assert_eq!(
      authority.resolve_access_message(
        plugin_name,
        command_allowed_on_webview_window,
        window,
        webview,
        &Origin::Remote {
          url: "http://localhost:123".parse().unwrap()
        }
      ),
      "myplugin.my-command-webview-window not allowed on window \"main-*\", webview \"webview-*\", URL: http://localhost:123/\n\nallowed on: [windows: \"main-*\", webviews: \"webview-*\", URL: local], [windows: \"main-*\", webviews: \"webview-*\", URL: http://localhost:8080]\n\nreferenced by: capability: maincap, permission: allow-command || capability: maincap, permission: allow-command"
    );
  }
}
