// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Resolved ACL for runtime usage.

use std::{
  collections::{hash_map::DefaultHasher, BTreeMap, HashSet},
  hash::{Hash, Hasher},
};

use crate::platform::Target;

use super::{
  capability::{Capability, CapabilityContext},
  plugin::Manifest,
  Error, ExecutionContext, Permission, PermissionSet, Value,
};

/// A key for a scope, used to link a [`ResolvedCommand#structfield.scope`] to the store [`Resolved#structfield.scopes`].
pub type ScopeKey = usize;

/// A resolved command permission.
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
  /// The list of window label patterns that is allowed to run this command.
  pub windows: Vec<glob::Pattern>,
  /// The reference of the scope that is associated with this command. See [`Resolved#structfield.scopes`].
  pub scope: Option<ScopeKey>,
}

/// A resolved scope. Merges all scopes defined for a single command.
#[derive(Debug, Default)]
pub struct ResolvedScope {
  /// Allows something on the command.
  pub allow: Vec<Value>,
  /// Denies something on the command.
  pub deny: Vec<Value>,
}

/// A command key for the map of allowed and denied commands.
/// Takes into consideration the command name and the execution context.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct CommandKey {
  /// The full command name.
  pub name: String,
  /// The context of the command.
  pub context: ExecutionContext,
}

/// Resolved access control list.
#[derive(Debug)]
pub struct Resolved {
  /// The commands that are allowed. Map each command with its context to a [`ResolvedCommand`].
  pub allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  /// The commands that are denied. Map each command with its context to a [`ResolvedCommand`].
  pub denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
  /// The store of scopes referenced by a [`ResolvedCommand`].
  pub command_scope: BTreeMap<ScopeKey, ResolvedScope>,
  /// The global scope.
  pub global_scope: ResolvedScope,
}

impl Resolved {
  /// Resolves the ACL for the given plugin permissions and app capabilities.
  pub fn resolve(
    acl: BTreeMap<String, Manifest>,
    capabilities: BTreeMap<String, Capability>,
    target: Target,
  ) -> Result<Self, Error> {
    let mut allowed_commands = BTreeMap::new();
    let mut denied_commands = BTreeMap::new();

    let mut current_scope_id = 0;
    let mut command_scopes = BTreeMap::new();
    let mut global_scope = Vec::new();

    // resolve commands
    for capability in capabilities.values() {
      if !capability.platforms.contains(&target) {
        continue;
      }

      for permission_id in &capability.permissions {
        let permission_name = permission_id.get_base();

        if let Some(plugin_name) = permission_id.get_prefix() {
          let permissions = get_permissions(plugin_name, permission_name, &acl)?;

          for permission in permissions {
            if permission.commands.allow.is_empty() && permission.commands.deny.is_empty() {
              // global scope
              global_scope.push(permission.scope.clone());
            } else {
              let has_scope = permission.scope.allow.is_some() || permission.scope.deny.is_some();
              if has_scope {
                current_scope_id += 1;
                command_scopes.insert(current_scope_id, permission.scope.clone());
              }

              let scope_id = if has_scope {
                Some(current_scope_id)
              } else {
                None
              };

              for allowed_command in &permission.commands.allow {
                resolve_command(
                  &mut allowed_commands,
                  format!("plugin:{plugin_name}|{allowed_command}"),
                  capability,
                  scope_id,
                );
              }

              for denied_command in &permission.commands.deny {
                resolve_command(
                  &mut denied_commands,
                  format!("plugin:{plugin_name}|{denied_command}"),
                  capability,
                  scope_id,
                );
              }
            }
          }
        }
      }
    }

    // resolve scopes
    let mut resolved_scopes = BTreeMap::new();

    for allowed in allowed_commands.values_mut() {
      if !allowed.scope.is_empty() {
        allowed.scope.sort();

        let mut hasher = DefaultHasher::new();
        allowed.scope.hash(&mut hasher);
        let hash = hasher.finish() as usize;

        allowed.resolved_scope_key.replace(hash);

        let resolved_scope = ResolvedScope {
          allow: allowed
            .scope
            .iter()
            .flat_map(|s| command_scopes.get(s).unwrap().allow.clone())
            .flatten()
            .collect(),
          deny: allowed
            .scope
            .iter()
            .flat_map(|s| command_scopes.get(s).unwrap().deny.clone())
            .flatten()
            .collect(),
        };

        resolved_scopes.insert(hash, resolved_scope);
      }
    }

    let global_scope = ResolvedScope {
      allow: global_scope
        .iter_mut()
        .flat_map(|s| s.allow.take())
        .flatten()
        .collect(),
      deny: global_scope
        .iter_mut()
        .flat_map(|s| s.deny.take())
        .flatten()
        .collect(),
    };

    let resolved = Self {
      allowed_commands: allowed_commands
        .into_iter()
        .map(|(key, cmd)| {
          Ok((
            key,
            ResolvedCommand {
              windows: parse_window_patterns(cmd.windows)?,
              scope: cmd.resolved_scope_key,
            },
          ))
        })
        .collect::<Result<_, Error>>()?,
      denied_commands: denied_commands
        .into_iter()
        .map(|(key, cmd)| {
          Ok((
            key,
            ResolvedCommand {
              windows: parse_window_patterns(cmd.windows)?,
              scope: cmd.resolved_scope_key,
            },
          ))
        })
        .collect::<Result<_, Error>>()?,
      command_scope: resolved_scopes,
      global_scope,
    };

    Ok(resolved)
  }
}

fn parse_window_patterns(windows: HashSet<String>) -> Result<Vec<glob::Pattern>, Error> {
  let mut patterns = Vec::new();
  for window in windows {
    patterns.push(glob::Pattern::new(&window)?);
  }
  Ok(patterns)
}

#[derive(Debug, Default)]
struct ResolvedCommandTemp {
  pub windows: HashSet<String>,
  pub scope: Vec<usize>,
  pub resolved_scope_key: Option<usize>,
}

fn resolve_command(
  commands: &mut BTreeMap<CommandKey, ResolvedCommandTemp>,
  command: String,
  capability: &Capability,
  scope_id: Option<usize>,
) {
  let contexts = match &capability.context {
    CapabilityContext::Local => {
      vec![ExecutionContext::Local]
    }
    CapabilityContext::Remote { domains } => domains
      .iter()
      .map(|domain| ExecutionContext::Remote {
        domain: domain.to_string(),
      })
      .collect(),
  };

  for context in contexts {
    let resolved = commands
      .entry(CommandKey {
        name: command.clone(),
        context,
      })
      .or_default();

    resolved.windows.extend(capability.windows.clone());
    if let Some(id) = scope_id {
      resolved.scope.push(id);
    }
  }
}

// get the permissions from a permission set
fn get_permission_set_permissions<'a>(
  manifest: &'a Manifest,
  set: &'a PermissionSet,
) -> Result<Vec<&'a Permission>, Error> {
  let mut permissions = Vec::new();

  for p in &set.permissions {
    if let Some(permission) = manifest.permissions.get(p) {
      permissions.push(permission);
    } else if let Some(permission_set) = manifest.permission_sets.get(p) {
      permissions.extend(get_permission_set_permissions(manifest, permission_set)?);
    } else {
      return Err(Error::SetPermissionNotFound {
        permission: p.to_string(),
        set: set.identifier.clone(),
      });
    }
  }

  Ok(permissions)
}

fn get_permissions<'a>(
  plugin_name: &'a str,
  permission_name: &'a str,
  acl: &'a BTreeMap<String, Manifest>,
) -> Result<Vec<&'a Permission>, Error> {
  let manifest = acl.get(plugin_name).ok_or_else(|| Error::UnknownPlugin {
    plugin: plugin_name.to_string(),
    available: acl.keys().cloned().collect::<Vec<_>>().join(", "),
  })?;

  if permission_name == "default" {
    manifest
      .default_permission
      .as_ref()
      .ok_or_else(|| Error::UnknownPermission {
        plugin: plugin_name.to_string(),
        permission: permission_name.to_string(),
      })
      .and_then(|default| get_permission_set_permissions(manifest, default))
  } else if let Some(set) = manifest.permission_sets.get(permission_name) {
    get_permission_set_permissions(manifest, set)
  } else if let Some(permission) = manifest.permissions.get(permission_name) {
    Ok(vec![permission])
  } else {
    Err(Error::UnknownPermission {
      plugin: plugin_name.to_string(),
      permission: permission_name.to_string(),
    })
  }
}

#[cfg(feature = "build")]
mod build {
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};
  use std::convert::identity;

  use super::*;
  use crate::tokens::*;

  /// Write a `TokenStream` of the `$struct`'s fields to the `$tokens`.
  ///
  /// All fields must represent a binding of the same name that implements `ToTokens`.
  macro_rules! literal_struct {
    ($tokens:ident, $struct:ident, $($field:ident),+) => {
      $tokens.append_all(quote! {
        ::tauri::utils::acl::resolved::$struct {
          $($field: #$field),+
        }
      })
    };
  }

  impl ToTokens for CommandKey {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let name = str_lit(&self.name);
      let context = &self.context;
      literal_struct!(tokens, CommandKey, name, context)
    }
  }

  impl ToTokens for ResolvedCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let windows = vec_lit(&self.windows, |window| {
        let w = window.as_str();
        quote!(#w.parse().unwrap())
      });
      let scope = opt_lit(self.scope.as_ref());
      literal_struct!(tokens, ResolvedCommand, windows, scope)
    }
  }

  impl ToTokens for ResolvedScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allow = vec_lit(&self.allow, identity);
      let deny = vec_lit(&self.deny, identity);
      literal_struct!(tokens, ResolvedScope, allow, deny)
    }
  }

  impl ToTokens for Resolved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allowed_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.allowed_commands,
        identity,
        identity,
      );

      let denied_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.denied_commands,
        identity,
        identity,
      );

      let command_scope = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.command_scope,
        identity,
        identity,
      );

      let global_scope = &self.global_scope;

      literal_struct!(
        tokens,
        Resolved,
        allowed_commands,
        denied_commands,
        command_scope,
        global_scope
      )
    }
  }
}
