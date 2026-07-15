// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Resolved ACL for runtime usage.

use std::{borrow::Cow, collections::BTreeMap, fmt};

use crate::platform::Target;

use super::{
  APP_ACL_KEY, Commands, Error, ExecutionContext, Identifier, Permission, PermissionSet, Scopes,
  Value,
  capability::{Capability, PermissionEntry},
  has_app_manifest,
  manifest::Manifest,
};

/// A key for a scope, used to link a [`ResolvedCommand#structfield.scope`] to the store [`Resolved#structfield.scopes`].
pub type ScopeKey = u64;

// All the fields are marked with `#[cfg(debug_assertions)]` but not the struct itself is because
// we want to avoid compilation errors on different `debug_assertions` settings,
// see https://github.com/tauri-apps/tauri/issues/13865
/// Metadata for what referenced a [`ResolvedCommand`].
#[derive(Default, Clone, PartialEq, Eq)]
pub struct ResolvedCommandReference {
  /// Identifier of the capability.
  #[cfg(debug_assertions)]
  pub capability: String,
  /// Identifier of the permission.
  #[cfg(debug_assertions)]
  pub permission: String,
}

impl ResolvedCommandReference {
  /// Internal helper for tauri-macros to avoid compilation errors on different `debug_assertions` settings,
  /// see https://github.com/tauri-apps/tauri/issues/13865
  #[doc(hidden)]
  pub fn new(
    #[cfg_attr(not(debug_assertions), allow(unused))] capability: String,
    #[cfg_attr(not(debug_assertions), allow(unused))] permission: String,
  ) -> Self {
    Self {
      #[cfg(debug_assertions)]
      capability,
      #[cfg(debug_assertions)]
      permission,
    }
  }
}

/// A resolved command permission.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct ResolvedCommand {
  /// The execution context of this command.
  pub context: ExecutionContext,
  /// The capability/permission that referenced this command.
  #[cfg(debug_assertions)]
  pub referenced_by: ResolvedCommandReference,
  /// The list of window label patterns that was resolved for this command.
  pub windows: Vec<glob::Pattern>,
  /// The list of webview label patterns that was resolved for this command.
  pub webviews: Vec<glob::Pattern>,
  /// The reference of the scope that is associated with this command. See [`Resolved#structfield.command_scopes`].
  pub scope_id: Option<ScopeKey>,
}

impl ResolvedCommand {
  /// Internal helper for tauri-macros to avoid compilation errors on different `debug_assertions` settings,
  /// see https://github.com/tauri-apps/tauri/issues/13865
  #[doc(hidden)]
  pub fn new(
    context: ExecutionContext,
    #[cfg_attr(not(debug_assertions), allow(unused))] referenced_by: ResolvedCommandReference,
    windows: Vec<glob::Pattern>,
    webviews: Vec<glob::Pattern>,
    scope_id: Option<ScopeKey>,
  ) -> Self {
    Self {
      context,
      #[cfg(debug_assertions)]
      referenced_by,
      windows,
      webviews,
      scope_id,
    }
  }
}

impl fmt::Debug for ResolvedCommand {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ResolvedCommand")
      .field("context", &self.context)
      .field("windows", &self.windows)
      .field("webviews", &self.webviews)
      .field("scope_id", &self.scope_id)
      .finish()
  }
}

/// A resolved scope. Merges all scopes defined for a single command.
#[derive(Debug, Default, Clone)]
pub struct ResolvedScope {
  /// Allows something on the command.
  pub allow: Vec<Value>,
  /// Denies something on the command.
  pub deny: Vec<Value>,
}

/// Resolved access control list.
#[derive(Debug, Default)]
pub struct Resolved {
  /// If we should check the ACL for the app commands
  pub has_app_acl: bool,
  /// The commands that are allowed. Map each command with its context to a [`ResolvedCommand`].
  pub allowed_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  /// The commands that are denied. Map each command with its context to a [`ResolvedCommand`].
  pub denied_commands: BTreeMap<String, Vec<ResolvedCommand>>,
  /// The store of scopes referenced by a [`ResolvedCommand`].
  pub command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  /// The global scope.
  pub global_scope: BTreeMap<String, ResolvedScope>,
}

impl Resolved {
  /// Resolves the ACL for the given plugin permissions and app capabilities.
  pub fn resolve(
    acl: &BTreeMap<String, Manifest>,
    mut capabilities: BTreeMap<String, Capability>,
    target: Target,
  ) -> Result<Self, Error> {
    let mut allowed_commands = BTreeMap::new();
    let mut denied_commands = BTreeMap::new();

    let mut current_scope_id = 0;
    let mut command_scope = BTreeMap::new();
    let mut global_scope: BTreeMap<String, Vec<Scopes>> = BTreeMap::new();

    // resolve commands
    for capability in capabilities.values_mut().filter(|c| c.is_active(&target)) {
      with_resolved_permissions(
        capability,
        acl,
        target,
        |ResolvedPermission {
           key,
           commands,
           scope,
           #[cfg_attr(not(debug_assertions), allow(unused))]
           permission_name,
         }| {
          if commands.allow.is_empty() && commands.deny.is_empty() {
            // global scope
            global_scope.entry(key.to_string()).or_default().push(scope);
          } else {
            let scope_id = if scope.allow.is_some() || scope.deny.is_some() {
              current_scope_id += 1;
              command_scope.insert(
                current_scope_id,
                ResolvedScope {
                  allow: scope.allow.unwrap_or_default(),
                  deny: scope.deny.unwrap_or_default(),
                },
              );
              Some(current_scope_id)
            } else {
              None
            };

            for allowed_command in &commands.allow {
              resolve_command(
                &mut allowed_commands,
                if key == APP_ACL_KEY {
                  allowed_command.to_string()
                } else if let Some(core_plugin_name) = key.strip_prefix("core:") {
                  format!("plugin:{core_plugin_name}|{allowed_command}")
                } else {
                  format!("plugin:{key}|{allowed_command}")
                },
                capability,
                scope_id,
                #[cfg(debug_assertions)]
                permission_name.to_string(),
              )?;
            }

            for denied_command in &commands.deny {
              resolve_command(
                &mut denied_commands,
                if key == APP_ACL_KEY {
                  denied_command.to_string()
                } else if let Some(core_plugin_name) = key.strip_prefix("core:") {
                  format!("plugin:{core_plugin_name}|{denied_command}")
                } else {
                  format!("plugin:{key}|{denied_command}")
                },
                capability,
                scope_id,
                #[cfg(debug_assertions)]
                permission_name.to_string(),
              )?;
            }
          }

          Ok(())
        },
      )?;
    }

    let global_scope = global_scope
      .into_iter()
      .map(|(key, scopes)| {
        let mut resolved_scope = ResolvedScope {
          allow: Vec::new(),
          deny: Vec::new(),
        };
        for scope in scopes {
          if let Some(allow) = scope.allow {
            resolved_scope.allow.extend(allow);
          }
          if let Some(deny) = scope.deny {
            resolved_scope.deny.extend(deny);
          }
        }
        (key, resolved_scope)
      })
      .collect();

    let resolved = Self {
      has_app_acl: has_app_manifest(acl),
      allowed_commands,
      denied_commands,
      command_scope,
      global_scope,
    };

    Ok(resolved)
  }
}

fn parse_glob_patterns(mut raw: Vec<String>) -> Result<Vec<glob::Pattern>, Error> {
  raw.sort();

  let mut patterns = Vec::new();
  for pattern in raw {
    patterns.push(glob::Pattern::new(&pattern)?);
  }

  Ok(patterns)
}

fn resolve_command(
  commands: &mut BTreeMap<String, Vec<ResolvedCommand>>,
  command: String,
  capability: &Capability,
  scope_id: Option<ScopeKey>,
  #[cfg(debug_assertions)] referenced_by_permission_identifier: String,
) -> Result<(), Error> {
  let mut contexts = Vec::new();
  if capability.local {
    contexts.push(ExecutionContext::Local);
  }
  if let Some(remote) = &capability.remote {
    contexts.extend(remote.urls.iter().map(|url| {
      ExecutionContext::Remote {
        url: url
          .parse()
          .unwrap_or_else(|e| panic!("invalid URL pattern for remote URL {url}: {e}")),
      }
    }));
  }

  for context in contexts {
    let resolved_list = commands.entry(command.clone()).or_default();

    resolved_list.push(ResolvedCommand {
      context,
      #[cfg(debug_assertions)]
      referenced_by: ResolvedCommandReference {
        capability: capability.identifier.clone(),
        permission: referenced_by_permission_identifier.clone(),
      },
      windows: parse_glob_patterns(capability.windows.clone())?,
      webviews: parse_glob_patterns(capability.webviews.clone())?,
      scope_id,
    });
  }

  Ok(())
}

struct ResolvedPermission<'a> {
  key: &'a str,
  permission_name: &'a str,
  commands: Commands,
  scope: Scopes,
}

/// Iterate over permissions in a capability, resolving permission sets if necessary
/// to produce a [`ResolvedPermission`] and calling the provided callback with it.
fn with_resolved_permissions<F: FnMut(ResolvedPermission<'_>) -> Result<(), Error>>(
  capability: &Capability,
  acl: &BTreeMap<String, Manifest>,
  target: Target,
  mut f: F,
) -> Result<(), Error> {
  for permission_entry in &capability.permissions {
    let permission_id = permission_entry.identifier();

    let permissions = get_permissions(permission_id, acl)?
      .into_iter()
      .filter(|p| p.permission.is_active(&target));

    for TraversedPermission {
      key,
      permission_name,
      permission,
    } in permissions
    {
      let mut resolved_scope = Scopes::default();
      let mut commands = Commands::default();

      if let PermissionEntry::ExtendedPermission {
        identifier: _,
        scope,
      } = permission_entry
      {
        if let Some(allow) = scope.allow.clone() {
          resolved_scope
            .allow
            .get_or_insert_with(Default::default)
            .extend(allow);
        }
        if let Some(deny) = scope.deny.clone() {
          resolved_scope
            .deny
            .get_or_insert_with(Default::default)
            .extend(deny);
        }
      }

      if let Some(allow) = permission.scope.allow.clone() {
        resolved_scope
          .allow
          .get_or_insert_with(Default::default)
          .extend(allow);
      }
      if let Some(deny) = permission.scope.deny.clone() {
        resolved_scope
          .deny
          .get_or_insert_with(Default::default)
          .extend(deny);
      }

      commands.allow.extend(permission.commands.allow.clone());
      commands.deny.extend(permission.commands.deny.clone());

      f(ResolvedPermission {
        key: &key,
        permission_name: &permission_name,
        commands,
        scope: resolved_scope,
      })?;
    }
  }

  Ok(())
}

/// Traversed permission
#[derive(Debug)]
pub struct TraversedPermission<'a> {
  /// Plugin name without the tauri-plugin- prefix
  pub key: String,
  /// Permission's name
  pub permission_name: String,
  /// Permission details.
  ///
  /// This is borrowed for permissions stored in the [`Manifest`], or owned for the `allow-$command`
  /// and `deny-$command` permissions [materialized on demand](Manifest::command_permission).
  pub permission: Cow<'a, Permission>,
}

/// Expand a permissions id based on the ACL to get the associated permissions (e.g. expand some-plugin:default)
pub fn get_permissions<'a>(
  permission_id: &Identifier,
  acl: &'a BTreeMap<String, Manifest>,
) -> Result<Vec<TraversedPermission<'a>>, Error> {
  let key = permission_id.get_prefix().unwrap_or(APP_ACL_KEY);
  let permission_name = permission_id.get_base();

  let manifest = acl.get(key).ok_or_else(|| Error::UnknownManifest {
    key: display_perm_key(key).to_string(),
    available: acl.keys().cloned().collect::<Vec<_>>().join(", "),
  })?;

  if permission_name == "default" {
    manifest
      .default_permission
      .as_ref()
      .map(|default| get_permission_set_permissions(permission_id, acl, manifest, default))
      .unwrap_or_else(|| Ok(Default::default()))
  } else if let Some(set) = manifest.permission_sets.get(permission_name) {
    get_permission_set_permissions(permission_id, acl, manifest, set)
  } else if let Some(permission) = manifest.permissions.get(permission_name) {
    Ok(vec![TraversedPermission {
      key: key.to_string(),
      permission_name: permission_name.to_string(),
      permission: Cow::Borrowed(permission),
    }])
  } else if let Some(permission) = manifest.command_permission(permission_name, key == APP_ACL_KEY)
  {
    Ok(vec![TraversedPermission {
      key: key.to_string(),
      permission_name: permission_name.to_string(),
      permission: Cow::Owned(permission),
    }])
  } else {
    Err(Error::UnknownPermission {
      key: display_perm_key(key).to_string(),
      permission: permission_name.to_string(),
    })
  }
}

// get the permissions from a permission set
fn get_permission_set_permissions<'a>(
  permission_id: &Identifier,
  acl: &'a BTreeMap<String, Manifest>,
  manifest: &'a Manifest,
  set: &'a PermissionSet,
) -> Result<Vec<TraversedPermission<'a>>, Error> {
  let key = permission_id.get_prefix().unwrap_or(APP_ACL_KEY);

  let mut permissions = Vec::new();

  for perm in &set.permissions {
    // a set could include permissions from other plugins
    // for example `dialog:default`, could include `fs:default`
    // in this case `perm = "fs:default"` which is not a permission
    // in the dialog manifest so we check if `perm` still have a prefix (i.e `fs:`)
    // and if so, we resolve this prefix from `acl` first before proceeding
    let id = Identifier::try_from(perm.clone()).expect("invalid identifier in permission set?");
    let (manifest, permission_id, key, permission_name) =
      if let Some((new_key, manifest)) = id.get_prefix().and_then(|k| acl.get(k).map(|m| (k, m))) {
        (manifest, &id, new_key, id.get_base())
      } else {
        (manifest, permission_id, key, perm.as_str())
      };

    if permission_name == "default" {
      permissions.extend(
        manifest
          .default_permission
          .as_ref()
          .map(|default| get_permission_set_permissions(permission_id, acl, manifest, default))
          .transpose()?
          .unwrap_or_default(),
      );
    } else if let Some(permission) = manifest.permissions.get(permission_name) {
      permissions.push(TraversedPermission {
        key: key.to_string(),
        permission_name: permission_name.to_string(),
        permission: Cow::Borrowed(permission),
      });
    } else if let Some(permission_set) = manifest.permission_sets.get(permission_name) {
      permissions.extend(get_permission_set_permissions(
        permission_id,
        acl,
        manifest,
        permission_set,
      )?);
    } else if let Some(permission) =
      manifest.command_permission(permission_name, key == APP_ACL_KEY)
    {
      permissions.push(TraversedPermission {
        key: key.to_string(),
        permission_name: permission_name.to_string(),
        permission: Cow::Owned(permission),
      });
    } else {
      return Err(Error::SetPermissionNotFound {
        permission: permission_name.to_string(),
        set: set.identifier.clone(),
      });
    }
  }

  Ok(permissions)
}

#[inline]
fn display_perm_key(prefix: &str) -> &str {
  if prefix == APP_ACL_KEY {
    "app manifest"
  } else {
    prefix
  }
}

#[cfg(any(feature = "build", feature = "build-2"))]
mod build {
  use proc_macro2::TokenStream;
  use quote::{ToTokens, TokenStreamExt, quote};
  use std::convert::identity;

  use super::*;
  use crate::{literal_struct, tokens::*};

  #[cfg(debug_assertions)]
  impl ToTokens for ResolvedCommandReference {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let capability = str_lit(&self.capability);
      let permission = str_lit(&self.permission);
      tokens.append_all(quote! {
        ::tauri::utils::acl::resolved::ResolvedCommandReference::new(#capability, #permission)
      });
    }
  }

  impl ToTokens for ResolvedCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      #[cfg(debug_assertions)]
      let referenced_by = &self.referenced_by;
      #[cfg(not(debug_assertions))]
      let referenced_by =
        quote!(::tauri::utils::acl::resolved::ResolvedCommandReference::default());

      let context = &self.context;

      let windows = vec_lit(&self.windows, |window| {
        let w = window.as_str();
        quote!(#w.parse().unwrap())
      });
      let webviews = vec_lit(&self.webviews, |window| {
        let w = window.as_str();
        quote!(#w.parse().unwrap())
      });
      let scope_id = opt_lit(self.scope_id.as_ref());

      tokens.append_all(quote! {
        ::tauri::utils::acl::resolved::ResolvedCommand::new(
          #context,
          #referenced_by,
          #windows,
          #webviews,
          #scope_id
        )
      })
    }
  }

  impl ToTokens for ResolvedScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allow = vec_lit(&self.allow, identity);
      let deny = vec_lit(&self.deny, identity);
      literal_struct!(
        tokens,
        ::tauri::utils::acl::resolved::ResolvedScope,
        allow,
        deny
      )
    }
  }

  impl ToTokens for Resolved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let has_app_acl = self.has_app_acl;

      let allowed_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.allowed_commands,
        str_lit,
        |v| vec_lit(v, identity),
      );

      let denied_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.denied_commands,
        str_lit,
        |v| vec_lit(v, identity),
      );

      let command_scope = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.command_scope,
        identity,
        identity,
      );

      let global_scope = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.global_scope,
        str_lit,
        identity,
      );

      literal_struct!(
        tokens,
        ::tauri::utils::acl::resolved::Resolved,
        has_app_acl,
        allowed_commands,
        denied_commands,
        command_scope,
        global_scope
      )
    }
  }
}

#[cfg(test)]
mod tests {

  use super::{
    APP_ACL_KEY, Identifier, Manifest, Permission, PermissionSet, Resolved, get_permissions,
  };
  use crate::platform::Target;

  fn manifest<const P: usize, const S: usize>(
    name: &str,
    permissions: [&str; P],
    default_set: Option<&[&str]>,
    sets: [(&str, &[&str]); S],
  ) -> (String, Manifest) {
    (
      name.to_string(),
      Manifest {
        default_permission: default_set.map(|perms| PermissionSet {
          identifier: "default".to_string(),
          description: "default set".to_string(),
          permissions: perms.iter().map(|s| s.to_string()).collect(),
        }),
        permissions: permissions
          .iter()
          .map(|p| {
            (
              p.to_string(),
              Permission {
                identifier: p.to_string(),
                ..Default::default()
              },
            )
          })
          .collect(),
        permission_sets: sets
          .iter()
          .map(|(s, perms)| {
            (
              s.to_string(),
              PermissionSet {
                identifier: s.to_string(),
                description: format!("{s} set"),
                permissions: perms.iter().map(|s| s.to_string()).collect(),
              },
            )
          })
          .collect(),
        ..Default::default()
      },
    )
  }

  fn id(id: &str) -> Identifier {
    Identifier::try_from(id.to_string()).unwrap()
  }

  #[test]
  fn resolves_permissions_from_other_plugins() {
    let acl = [
      manifest(
        "fs",
        ["read", "write", "rm", "exist"],
        Some(&["read", "exist"]),
        [],
      ),
      manifest(
        "http",
        ["fetch", "fetch-cancel"],
        None,
        [("fetch-with-cancel", &["fetch", "fetch-cancel"])],
      ),
      manifest(
        "dialog",
        ["open", "save"],
        None,
        [(
          "extra",
          &[
            "save",
            "fs:default",
            "fs:write",
            "http:default",
            "http:fetch-with-cancel",
          ],
        )],
      ),
    ]
    .into();

    let permissions = get_permissions(&id("fs:default"), &acl).unwrap();
    assert_eq!(permissions.len(), 2);
    assert_eq!(permissions[0].key, "fs");
    assert_eq!(permissions[0].permission_name, "read");
    assert_eq!(permissions[1].key, "fs");
    assert_eq!(permissions[1].permission_name, "exist");

    let permissions = get_permissions(&id("fs:rm"), &acl).unwrap();
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].key, "fs");
    assert_eq!(permissions[0].permission_name, "rm");

    let permissions = get_permissions(&id("http:fetch-with-cancel"), &acl).unwrap();
    assert_eq!(permissions.len(), 2);
    assert_eq!(permissions[0].key, "http");
    assert_eq!(permissions[0].permission_name, "fetch");
    assert_eq!(permissions[1].key, "http");
    assert_eq!(permissions[1].permission_name, "fetch-cancel");

    let permissions = get_permissions(&id("dialog:extra"), &acl).unwrap();
    assert_eq!(permissions.len(), 6);
    assert_eq!(permissions[0].key, "dialog");
    assert_eq!(permissions[0].permission_name, "save");
    assert_eq!(permissions[1].key, "fs");
    assert_eq!(permissions[1].permission_name, "read");
    assert_eq!(permissions[2].key, "fs");
    assert_eq!(permissions[2].permission_name, "exist");
    assert_eq!(permissions[3].key, "fs");
    assert_eq!(permissions[3].permission_name, "write");
    assert_eq!(permissions[4].key, "http");
    assert_eq!(permissions[4].permission_name, "fetch");
    assert_eq!(permissions[5].key, "http");
    assert_eq!(permissions[5].permission_name, "fetch-cancel");
  }

  fn manifest_with_commands(name: &str, commands: &[&str]) -> (String, Manifest) {
    (
      name.to_string(),
      Manifest {
        commands: commands.iter().map(|c| c.to_string()).collect(),
        ..Default::default()
      },
    )
  }

  #[test]
  fn resolves_implicit_command_permissions() {
    let acl = [manifest_with_commands("fs", &["read_file", "write_file"])].into();

    // `allow-$command` resolves to the command with the original (snake_case) name
    let permissions = get_permissions(&id("fs:allow-read-file"), &acl).unwrap();
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].key, "fs");
    assert_eq!(permissions[0].permission.commands.allow, ["read_file"]);
    assert!(permissions[0].permission.commands.deny.is_empty());

    // `deny-$command` resolves to the deny side
    let permissions = get_permissions(&id("fs:deny-write-file"), &acl).unwrap();
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].permission.commands.deny, ["write_file"]);

    // unknown command is still an error
    assert!(get_permissions(&id("fs:allow-unknown"), &acl).is_err());
  }

  #[test]
  fn resolves_wildcard_command_permission() {
    let acl = [(
      APP_ACL_KEY.to_string(),
      manifest_with_commands("__app__", &["read_file", "write_file"]).1,
    )]
    .into();

    // the wildcard resolves to a single `*` command instead of one entry per command
    let permissions = get_permissions(&id("allow-*"), &acl).unwrap();
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].permission.commands.allow, ["*"]);

    let permissions = get_permissions(&id("deny-*"), &acl).unwrap();
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].permission.commands.deny, ["*"]);

    // the wildcard is only available for the app manifest, not for plugins
    let plugin_acl = [manifest_with_commands("fs", &["read_file", "write_file"])].into();
    assert!(get_permissions(&id("fs:allow-*"), &plugin_acl).is_err());
    // but specific command permissions still work for plugins
    assert!(get_permissions(&id("fs:allow-read-file"), &plugin_acl).is_ok());

    // the wildcard is not a valid permission when the app manifest has no commands
    let empty = [(APP_ACL_KEY.to_string(), manifest("__app__", [], None, []).1)].into();
    assert!(get_permissions(&id("allow-*"), &empty).is_err());
  }

  #[test]
  fn resolve_wildcard_uses_single_entry() {
    use crate::acl::{Capability, capability::PermissionEntry};

    let acl = [(
      APP_ACL_KEY.to_string(),
      Manifest {
        commands: ["echo", "ping", "spam"]
          .iter()
          .map(|c| c.to_string())
          .collect(),
        ..Default::default()
      },
    )]
    .into();

    let capability = Capability {
      identifier: "main".to_string(),
      description: String::new(),
      remote: None,
      local: true,
      windows: vec!["main".to_string()],
      webviews: Vec::new(),
      permissions: vec![PermissionEntry::PermissionRef(id("allow-*"))],
      platforms: None,
    };

    let resolved = Resolved::resolve(
      &acl,
      [("main".to_string(), capability)].into(),
      Target::Linux,
    )
    .unwrap();

    // the whole app is allowed through a single resolved entry keyed by `*`
    assert_eq!(resolved.allowed_commands.len(), 1);
    assert!(resolved.allowed_commands.contains_key("*"));
    assert!(resolved.denied_commands.is_empty());
  }
}
