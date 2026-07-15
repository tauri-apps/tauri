// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex, OnceLock};

use serde::de::DeserializeOwned;

#[cfg(feature = "dynamic-acl")]
use tauri_utils::acl::capability::CapabilityFile;
#[cfg(any(feature = "dynamic-acl", debug_assertions))]
use tauri_utils::acl::manifest::Manifest;
use tauri_utils::acl::{
  APP_ACL_KEY, ExecutionContext, Value,
  resolved::{Resolved, ResolvedCommand, ResolvedScope, ScopeKey},
};

use url::Url;

use crate::{AppHandle, Manager, StateManager, Webview};
use crate::{Runtime, ipc::InvokeError, sealed::ManagerBase};

use super::{CommandArg, CommandItem};

/// Materialized authority data derived from the resolved ACL.
///
/// Built lazily (see [`RuntimeAuthority::inner`]) because constructing it is the dominant cost
/// of `generate_context!`: the resolved ACL (`allowed_commands` / `denied_commands` / scopes for
/// every command) is emitted by `tauri-codegen` as a large literal built at runtime. It is only
/// needed at command-dispatch time, never during startup, so building it off-thread keeps that
/// cost off the startup critical path. Deep-link forwards exit without ever dispatching a
/// command, so they never block on the build: it runs on the background thread and is discarded
/// when the process exits.
struct RuntimeAuthorityInner {
  /// Raw ACL manifests. Only read on the command-denied error path
  /// ([`RuntimeAuthority::resolve_access_message`]) and by the `dynamic-acl` feature; dropped
  /// entirely in release builds.
  #[cfg(any(feature = "dynamic-acl", debug_assertions))]
  acl: BTreeMap<String, Manifest>,
  has_app_acl: bool,
  allowed_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  denied_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  scope_manager: ScopeManager,
}

/// `Send` inputs to a [`RuntimeAuthorityInner`], produced off-thread by
/// [`RuntimeAuthority::new_async`].
///
/// Only the expensive, `Send` data (the resolved ACL and raw manifests) is built on the
/// background thread; the empty `StateManager` scope caches are assembled on the consuming
/// thread in [`RuntimeAuthority::build_inner`] (they are not `Send` and are cheap to create).
struct ResolvedAcl {
  #[cfg(any(feature = "dynamic-acl", debug_assertions))]
  acl: BTreeMap<String, Manifest>,
  resolved: Resolved,
}

/// The runtime authority used to authorize IPC execution based on the Access Control List.
pub struct RuntimeAuthority {
  /// Materialized authority data. Populated eagerly by [`Self::new`] or lazily by
  /// [`Self::inner`], which joins the background builder on first access.
  inner: OnceLock<RuntimeAuthorityInner>,
  /// Lazy builder for [`Self::inner`]. `None` for the eager [`Self::new`] constructor.
  ///
  /// For [`Self::new_async`] it holds a *deferred* builder that is not started until
  /// [`Self::begin_build`] spawns it — after the runtime is created — so ACL construction stays
  /// off the startup critical path without racing runtime init. [`Self::inner`] consumes it
  /// (joining the thread, or building inline if it was never spawned) on first access.
  build: Option<Mutex<Option<AclBuild>>>,
}

/// Lazy-build state for [`RuntimeAuthority::inner`], held in [`RuntimeAuthority::build`].
enum AclBuild {
  /// Builder stored but not yet running. Spawned by [`RuntimeAuthority::begin_build`] once the
  /// runtime exists, or built inline by [`RuntimeAuthority::inner`] if the authority is read
  /// first.
  Deferred(Box<dyn FnOnce() -> ResolvedAcl + Send + 'static>),
  /// Building on a background thread; joined by [`RuntimeAuthority::inner`].
  Building(std::thread::JoinHandle<ResolvedAcl>),
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

/// This is used internally by [`crate::generate_handler!`] for constructing [`RuntimeAuthority`]
/// to only include the raw ACL when it's needed
///
/// ## Stability
///
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[cfg(any(feature = "dynamic-acl", debug_assertions))]
#[doc(hidden)]
#[macro_export]
macro_rules! runtime_authority {
  ($acl:expr, $resolved_acl:expr) => {
    // Build the (expensive) resolved ACL and raw ACL on a background thread; the first
    // command-authorization check blocks on the result. `|| $acl` / `|| $resolved_acl` are
    // non-capturing (both are emitted as compile-time literals), so they coerce to `fn`
    // pointers. This keeps ACL construction — the dominant cost of `generate_context!` — off
    // the startup critical path. See `RuntimeAuthority`.
    $crate::ipc::RuntimeAuthority::new_async(|| $acl, || $resolved_acl)
  };
}

/// This is used internally by [`crate::generate_handler!`] for constructing [`RuntimeAuthority`]
/// to only include the raw ACL when it's needed
///
/// ## Stability
///
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[cfg(not(any(feature = "dynamic-acl", debug_assertions)))]
#[doc(hidden)]
#[macro_export]
macro_rules! runtime_authority {
  ($_acl:expr, $resolved_acl:expr) => {
    // Release builds drop the raw ACL entirely; only the resolved ACL is built, off-thread.
    $crate::ipc::RuntimeAuthority::new_async(|| $resolved_acl)
  };
}

impl RuntimeAuthority {
  /// Assembles [`RuntimeAuthorityInner`] from resolved ACL data.
  ///
  /// The expensive part (the [`Resolved`] literal) is already built; this only wires up the
  /// empty [`StateManager`] scope caches, so it is cheap and runs on the consuming thread.
  fn build_inner(resolved_acl: ResolvedAcl) -> RuntimeAuthorityInner {
    let ResolvedAcl {
      #[cfg(any(feature = "dynamic-acl", debug_assertions))]
      acl,
      resolved,
    } = resolved_acl;
    let command_cache = resolved
      .command_scope
      .keys()
      .map(|key| (*key, StateManager::new()))
      .collect();
    RuntimeAuthorityInner {
      #[cfg(any(feature = "dynamic-acl", debug_assertions))]
      acl,
      has_app_acl: resolved.has_app_acl,
      allowed_commands: resolved.allowed_commands,
      denied_commands: resolved.denied_commands,
      scope_manager: ScopeManager {
        command_scope: resolved.command_scope,
        global_scope: resolved.global_scope,
        command_cache,
        global_scope_cache: StateManager::new(),
      },
    }
  }

  /// Construct a new [`RuntimeAuthority`] from already-resolved ACL data (built eagerly).
  ///
  /// Prefer the [`runtime_authority`] macro, which builds the ACL lazily off-thread via
  /// [`Self::new_async`]. This eager constructor is for callers that already have the resolved
  /// ACL in hand (e.g. tests).
  #[doc(hidden)]
  pub fn new(
    #[cfg(any(feature = "dynamic-acl", debug_assertions))] acl: BTreeMap<String, Manifest>,
    resolved_acl: Resolved,
  ) -> Self {
    let inner = OnceLock::new();
    let _ = inner.set(Self::build_inner(ResolvedAcl {
      #[cfg(any(feature = "dynamic-acl", debug_assertions))]
      acl,
      resolved: resolved_acl,
    }));
    Self { inner, build: None }
  }

  /// Construct a new [`RuntimeAuthority`] whose resolved ACL is built lazily on a background
  /// thread.
  ///
  /// The resolved ACL (and raw ACL) is the dominant cost of `generate_context!`, yet it is only
  /// needed at command-dispatch time. Building it off-thread keeps it off the startup critical
  /// path; the first read via [`Self::inner`] (e.g. [`Self::resolve_access`]) blocks on the
  /// result. The [`runtime_authority`] macro passes non-capturing closures over compile-time
  /// literals; the `Send + 'static` bound also lets callers (e.g. tests) pass capturing
  /// closures.
  ///
  /// The builder is *stored, not spawned*: the background thread is started later by
  /// [`Self::begin_build`], once the runtime has been created. Spawning it eagerly here (at
  /// `generate_context!()` time) would let it allocate *during* runtime init, which is unsafe on
  /// runtimes that replace the process allocator in `new` — CEF loads the Chromium framework,
  /// swapping macOS's default `malloc` zone, and an allocation racing that swap corrupts the
  /// heap and crashes at startup. Deferring the spawn keeps the off-thread win (the build
  /// overlaps webview creation and frontend boot) without the race.
  ///
  /// The thread spawn + join is pure overhead for trivial ACLs (e.g. tests and small apps), so
  /// this only pays off above a non-trivial resolved-ACL size; the size is not known until the
  /// ACL is built, so the trade-off cannot be gated at runtime. Use [`Self::new`] when the
  /// resolved ACL is already in hand and the overhead is not worth it.
  ///
  /// **Please prefer using the [`runtime_authority`] macro instead of calling this directly**
  #[doc(hidden)]
  pub fn new_async(
    #[cfg(any(feature = "dynamic-acl", debug_assertions))] acl_builder: impl FnOnce() -> BTreeMap<
      String,
      Manifest,
    > + Send
    + 'static,
    resolved_builder: impl FnOnce() -> Resolved + Send + 'static,
  ) -> Self {
    let builder: Box<dyn FnOnce() -> ResolvedAcl + Send + 'static> =
      Box::new(move || ResolvedAcl {
        #[cfg(any(feature = "dynamic-acl", debug_assertions))]
        acl: acl_builder(),
        resolved: resolved_builder(),
      });
    Self {
      inner: OnceLock::new(),
      build: Some(Mutex::new(Some(AclBuild::Deferred(builder)))),
    }
  }

  /// Spawns the resolved-ACL builder on a dedicated background thread.
  fn spawn_builder(
    builder: Box<dyn FnOnce() -> ResolvedAcl + Send + 'static>,
  ) -> std::thread::JoinHandle<ResolvedAcl> {
    std::thread::Builder::new()
      .name(String::from("tauri runtime authority"))
      // The resolved-ACL literal construction is deep; give it the headroom the generated
      // context-creation thread used to need (it no longer constructs the ACL inline).
      .stack_size(8 * 1024 * 1024)
      .spawn(builder)
      .expect("failed to spawn runtime authority builder thread")
  }

  /// Starts the deferred resolved-ACL build on a background thread, if it has not started yet.
  ///
  /// Called once the runtime has been created (see [`crate::Builder::build`]). Deferring the
  /// spawn to here — rather than eagerly in [`Self::new_async`] — keeps ACL construction off the
  /// startup critical path (it overlaps webview creation and frontend boot) while guaranteeing
  /// no builder thread allocates *during* runtime init, which would race the allocator swap that
  /// CEF performs when it loads the Chromium framework and crash at startup. No-op for the eager
  /// [`Self::new`] constructor or once the build has started; [`Self::inner`] joins it on first
  /// access.
  pub(crate) fn begin_build(&self) {
    let Some(build) = self.build.as_ref() else {
      return;
    };
    let mut guard = build.lock().unwrap();
    match guard.take() {
      Some(AclBuild::Deferred(builder)) => {
        *guard = Some(AclBuild::Building(Self::spawn_builder(builder)));
      }
      // Already building, or already consumed by `inner()`: put it back untouched.
      other => *guard = other,
    }
  }

  /// Returns the materialized authority data, blocking on the background builder on first
  /// access if this authority was constructed via [`Self::new_async`].
  fn inner(&self) -> &RuntimeAuthorityInner {
    self.inner.get_or_init(|| {
      let build = self
        .build
        .as_ref()
        .expect("runtime authority has neither resolved data nor a builder")
        .lock()
        .unwrap()
        .take()
        // Taken exactly once. If it is already gone while `inner` is still unset, a previous
        // `inner()` call took it and panicked, so report that rather than the misleading
        // "neither data nor builder".
        .expect("runtime authority builder thread panicked");
      let handle = match build {
        AclBuild::Building(handle) => handle,
        // `begin_build` never ran (the authority was read before the runtime was created); build
        // it now on a big-stack thread, matching the deferred path's headroom.
        AclBuild::Deferred(builder) => Self::spawn_builder(builder),
      };
      let resolved_acl = handle
        .join()
        .expect("runtime authority builder thread panicked");
      Self::build_inner(resolved_acl)
    })
  }

  /// Mutable access to the materialized authority data, materializing it first if needed.
  fn inner_mut(&mut self) -> &mut RuntimeAuthorityInner {
    // Force materialization (the returned shared borrow ends immediately), then hand out `&mut`.
    let _ = self.inner();
    self
      .inner
      .get_mut()
      .expect("runtime authority materialized above")
  }

  /// The scope manager, materializing the authority data first if needed.
  pub(crate) fn scope_manager(&self) -> &ScopeManager {
    &self.inner().scope_manager
  }

  pub(crate) fn has_app_manifest(&self) -> bool {
    self.inner().has_app_acl
  }

  #[doc(hidden)]
  pub fn __allow_command(&mut self, command: String, context: ExecutionContext) {
    self.inner_mut().allowed_commands.insert(
      command,
      vec![ResolvedCommand {
        context,
        windows: vec!["*".parse().unwrap()],
        ..Default::default()
      }],
    );
  }

  /// Adds the given capability to the runtime authority.
  #[cfg(feature = "dynamic-acl")]
  pub fn add_capability(&mut self, capability: impl super::RuntimeCapability) -> crate::Result<()> {
    self.add_capability_inner(capability.build())
  }

  #[cfg(feature = "dynamic-acl")]
  fn add_capability_inner(&mut self, capability: CapabilityFile) -> crate::Result<()> {
    let mut capabilities = BTreeMap::new();
    match capability {
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

    // Resolve against the raw ACL (materializes the authority) before taking `&mut` below.
    let resolved = Resolved::resolve(
      &self.inner().acl,
      capabilities,
      tauri_utils::platform::Target::current(),
    )
    .unwrap();

    let inner = self.inner_mut();

    // fill global scope
    for (plugin, global_scope) in resolved.global_scope {
      let global_scope_entry = inner.scope_manager.global_scope.entry(plugin).or_default();

      global_scope_entry.allow.extend(global_scope.allow);
      global_scope_entry.deny.extend(global_scope.deny);

      inner.scope_manager.global_scope_cache = StateManager::new();
    }

    // denied commands
    for (cmd_key, resolved_cmds) in resolved.denied_commands {
      let entry = inner.denied_commands.entry(cmd_key).or_default();
      entry.extend(resolved_cmds);
    }

    // allowed commands
    for (cmd_key, resolved_cmds) in resolved.allowed_commands {
      // fill command scope
      for resolved_cmd in &resolved_cmds {
        if let Some(scope_id) = resolved_cmd.scope_id {
          let command_scope = resolved.command_scope.get(&scope_id).unwrap();

          let command_scope_entry = inner
            .scope_manager
            .command_scope
            .entry(scope_id)
            .or_default();
          command_scope_entry
            .allow
            .extend(command_scope.allow.clone());
          command_scope_entry.deny.extend(command_scope.deny.clone());

          inner
            .scope_manager
            .command_cache
            .insert(scope_id, StateManager::new());
        }
      }

      let entry = inner.allowed_commands.entry(cmd_key).or_default();
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
      manifest: &Manifest,
      set: &crate::utils::acl::PermissionSet,
      command: &str,
      allow_wildcard: bool,
    ) -> bool {
      for permission_id in &set.permissions {
        if permission_id == "default" {
          if let Some(default) = &manifest.default_permission
            && has_permissions_allowing_command(manifest, default, command, allow_wildcard)
          {
            return true;
          }
        } else if let Some(ref_set) = manifest.permission_sets.get(permission_id)
          && has_permissions_allowing_command(manifest, ref_set, command, allow_wildcard)
        {
          return true;
        } else if let Some(permission) = manifest.permissions.get(permission_id)
          && permission.commands.allow.contains(&command.into())
        {
          return true;
        } else if let Some(permission) = manifest.command_permission(permission_id, allow_wildcard)
        {
          // `*` is the wildcard command produced by the `allow-*` permission
          if permission
            .commands
            .allow
            .iter()
            .any(|c| c == command || c == "*")
          {
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

    // Materialize the authority once; everything below reads the resolved ACL.
    let inner = self.inner();

    if let Some(resolved) = inner.denied_commands.get(&command) {
      format!(
        "{command_pretty_name} explicitly denied on origin {origin}\n\nreferenced by: {}",
        print_references(resolved)
      )
    } else {
      let command_matches = inner.allowed_commands.get(&command);

      if let Some(resolved) = inner.allowed_commands.get(&command) {
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
          format!(
            "{command_pretty_name} not allowed on window \"{window}\", webview \"{webview}\", URL: {}\n\n{}\n\nreferenced by: {}",
            match origin {
              Origin::Local => "local",
              Origin::Remote { url } => url.as_str(),
            },
            print_allowed_on(resolved),
            print_references(resolved)
          )
        }
      } else {
        let permission_error_detail = if let Some((key, manifest)) = inner
          .acl
          .get_key_value(key)
          .or_else(|| inner.acl.get_key_value(&format!("core:{key}")))
        {
          let mut permissions_referencing_command = Vec::new();

          // the `allow-*`/`deny-*` wildcards are only available for the app manifest
          let allow_wildcard = key == APP_ACL_KEY;

          if let Some(default) = &manifest.default_permission
            && has_permissions_allowing_command(manifest, default, command_name, allow_wildcard)
          {
            permissions_referencing_command.push("default".into());
          }
          for set in manifest.permission_sets.values() {
            if has_permissions_allowing_command(manifest, set, command_name, allow_wildcard) {
              permissions_referencing_command.push(set.identifier.clone());
            }
          }
          for permission in manifest.permissions.values() {
            if permission.commands.allow.contains(&command_name.into()) {
              permissions_referencing_command.push(permission.identifier.clone());
            }
          }
          if manifest.commands.iter().any(|c| c == command_name) {
            permissions_referencing_command
              .push(format!("allow-{}", command_name.replace('_', "-")));
          }
          if allow_wildcard && !manifest.commands.is_empty() {
            permissions_referencing_command.push("allow-*".to_string());
          }

          permissions_referencing_command.sort();

          let associated_permissions = permissions_referencing_command
            .into_iter()
            .map(|permission| {
              if key == APP_ACL_KEY {
                permission
              } else {
                format!("{key}:{permission}")
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
            "{command_pretty_name} not allowed on origin [{origin}]. Please create a capability that has this origin on the context field.\n\nFound matches for: {}\n\n{permission_error_detail}",
            resolved_cmds
              .iter()
              .map(|resolved| {
                let context = match &resolved.context {
                  ExecutionContext::Local => "[local]".to_string(),
                  ExecutionContext::Remote { url } => format!("[remote: {}]", url.as_str()),
                };
                format!(
                  "- context: {context}, referenced by: capability: {}, permission: {}",
                  resolved.referenced_by.capability, resolved.referenced_by.permission
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
    // First command dispatch blocks here if the resolved ACL is still building on the
    // background thread (see `RuntimeAuthority::new_async`); subsequent calls are cached.
    let inner = self.inner();
    // the `allow-*`/`deny-*` wildcard permissions resolve to a single `*` command (per manifest)
    // instead of one entry per command, so we also look the command up under its wildcard key.
    let wildcard = wildcard_command(command);
    if inner
      .denied_commands
      .get(command)
      .or_else(|| inner.denied_commands.get(&wildcard))
      .map(|resolved| resolved.iter().any(|cmd| origin.matches(&cmd.context)))
      .is_some()
    {
      None
    } else {
      let resolved_cmds = inner
        .allowed_commands
        .get(command)
        .into_iter()
        .chain(inner.allowed_commands.get(&wildcard))
        .flatten()
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
    }
  }
}

/// The wildcard command key that matches every command of the same manifest:
/// `*` for app commands and `plugin:$name|*` for plugin commands.
///
/// Used to resolve the implicit `allow-*`/`deny-*` permissions without expanding them
/// into one entry per command in the resolved ACL.
fn wildcard_command(command: &str) -> String {
  match command.rsplit_once('|') {
    Some((prefix, _)) => format!("{prefix}|*"),
    None => "*".to_string(),
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
        .scope_manager()
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
      .scope_manager()
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
      Some(cached) => Ok((*cached).clone()),
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
      Some(cached) => Ok((*cached).clone()),
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
    ExecutionContext,
    resolved::{Resolved, ResolvedCommand},
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
  fn wildcard_command_allows_any_app_command() {
    let window = "main";
    let webview = "main";

    // a single `*` entry stands in for every app command (the `allow-*` permission)
    let resolved_cmd = vec![ResolvedCommand {
      windows: vec![Pattern::new(window).unwrap()],
      ..Default::default()
    }];
    let allowed_commands = [("*".to_string(), resolved_cmd.clone())]
      .into_iter()
      .collect();

    let authority = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands,
        ..Default::default()
      },
    );

    // an arbitrary app command (never listed explicitly) is allowed through the wildcard entry
    assert_eq!(
      authority.resolve_access("some_command", window, webview, &Origin::Local),
      Some(resolved_cmd.clone())
    );
    assert_eq!(
      authority.resolve_access("another_command", window, webview, &Origin::Local),
      Some(resolved_cmd)
    );

    // plugin commands use a per-plugin wildcard key, so the app wildcard does not allow them
    assert!(
      authority
        .resolve_access("plugin:fs|read", window, webview, &Origin::Local)
        .is_none()
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

    assert!(
      authority
        .resolve_access(
          command,
          window,
          webview,
          &Origin::Remote {
            url: "https://tauri.app".parse().unwrap()
          }
        )
        .is_none()
    );
  }

  #[test]
  fn denied_command_takes_precedence() {
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
        windows,
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

    assert!(
      authority
        .resolve_access(command, window, webview, &Origin::Local)
        .is_none()
    );
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
      referenced_by,
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
          commands: Default::default(),
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

  // ============================================================================
  // Async (background-built) authority
  // ============================================================================
  //
  // The `runtime_authority!` macro builds the resolved ACL on a background thread via
  // `new_async`; the first authorization read joins it (see `RuntimeAuthority::inner`). These
  // tests assert the background-built authority authorizes identically to the eager `new` path.

  /// A resolved ACL with one allowed command (window `main-*`, command scope `1`), one denied
  /// command (any window), and a command scope. Built fresh on each call so it can be used both
  /// eagerly and as a `new_async` builder.
  fn sample_resolved() -> Resolved {
    use tauri_utils::acl::resolved::{ResolvedScope, ScopeKey};

    let allowed_commands = [(
      "allowed-command".to_string(),
      vec![ResolvedCommand {
        windows: vec![Pattern::new("main-*").unwrap()],
        scope_id: Some(1 as ScopeKey),
        ..Default::default()
      }],
    )]
    .into_iter()
    .collect();
    let denied_commands = [(
      "denied-command".to_string(),
      vec![ResolvedCommand {
        windows: vec![Pattern::new("*").unwrap()],
        ..Default::default()
      }],
    )]
    .into_iter()
    .collect();
    let command_scope = [(1 as ScopeKey, ResolvedScope::default())]
      .into_iter()
      .collect();

    Resolved {
      allowed_commands,
      denied_commands,
      command_scope,
      ..Default::default()
    }
  }

  #[test]
  fn async_authority_resolves_allowed_command() {
    // Built off-thread; `resolve_access` must block on the builder, then authorize correctly.
    let authority = RuntimeAuthority::new_async(|| Default::default(), sample_resolved);

    assert!(
      authority
        .resolve_access("allowed-command", "main-window", "wv", &Origin::Local)
        .is_some(),
      "allowed command should resolve through the background-built authority"
    );
    assert!(
      authority
        .resolve_access("unknown-command", "main-window", "wv", &Origin::Local)
        .is_none(),
      "unknown command must not be allowed"
    );
  }

  #[test]
  fn async_authority_denied_takes_precedence() {
    let authority = RuntimeAuthority::new_async(|| Default::default(), sample_resolved);
    assert!(
      authority
        .resolve_access("denied-command", "anything", "wv", &Origin::Local)
        .is_none(),
      "denied command must be rejected through the background-built authority"
    );
  }

  #[test]
  fn async_and_eager_authority_agree() {
    let eager = RuntimeAuthority::new(Default::default(), sample_resolved());
    let lazy = RuntimeAuthority::new_async(|| Default::default(), sample_resolved);

    for (command, window) in [
      ("allowed-command", "main-1"),
      ("allowed-command", "other-1"),
      ("denied-command", "main-1"),
      ("unknown-command", "main-1"),
    ] {
      assert_eq!(
        eager.resolve_access(command, window, "wv", &Origin::Local),
        lazy.resolve_access(command, window, "wv", &Origin::Local),
        "eager and background-built authority disagree for command={command} window={window}"
      );
    }
  }

  #[test]
  fn async_authority_scope_manager_materializes() {
    // `scope_manager()` must also join the background builder and expose the resolved scopes.
    let authority = RuntimeAuthority::new_async(|| Default::default(), sample_resolved);
    assert!(
      authority.scope_manager().command_scope.contains_key(&1),
      "scope manager should expose the resolved command scope after materialization"
    );
  }

  #[cfg(debug_assertions)]
  #[test]
  fn async_authority_resolve_access_message_materializes() {
    // The debug-only error path reads the raw ACL through the background-built authority; ensure
    // it materializes and produces a denial message without panicking.
    let authority = RuntimeAuthority::new_async(|| Default::default(), sample_resolved);
    let message = authority.resolve_access_message(
      super::APP_ACL_KEY,
      "denied-command",
      "win",
      "wv",
      &Origin::Local,
    );
    assert!(
      message.contains("denied"),
      "expected a denial message, got: {message}"
    );
  }
}
