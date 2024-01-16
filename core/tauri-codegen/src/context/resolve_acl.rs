// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{hash_map::DefaultHasher, BTreeMap, HashMap},
  hash::{Hash, Hasher},
};

use tauri_utils::acl::{
  capability::{Capability, CapabilityContext},
  plugin::Manifest,
  resolved::{CommandKey, Resolved, ResolvedCommand, ResolvedScope},
  ExecutionContext, Permission, PermissionSet,
};

#[derive(Debug, Default)]
struct ResolvedCommandTemp {
  pub windows: Vec<String>,
  pub scope: Vec<usize>,
  pub resolved_scope_key: usize,
}

pub fn resolve(
  acl: HashMap<String, Manifest>,
  capabilities: HashMap<String, Capability>,
) -> Result<Resolved, Box<dyn std::error::Error>> {
  let mut allowed_commands = HashMap::new();
  let mut denied_commands = HashMap::new();

  let mut current_scope_id = 0;
  let mut command_scopes = HashMap::new();
  let mut global_scope = Vec::new();

  // resolve commands
  for capability in capabilities.values() {
    for permission_id in &capability.permissions {
      let permission_name = permission_id.get_base();

      if let Some(plugin_name) = permission_id.get_prefix() {
        let permissions = get_permissions(plugin_name, permission_name, &acl)?;

        for permission in permissions {
          if permission.commands.allow.is_empty() && permission.commands.deny.is_empty() {
            // global scope
            global_scope.push(permission.scope.clone());
          } else {
            current_scope_id += 1;
            command_scopes.insert(current_scope_id, permission.scope.clone());

            for allowed_command in &permission.commands.allow {
              resolve_command(
                &mut allowed_commands,
                format!("plugin:{plugin_name}|{allowed_command}"),
                capability,
                current_scope_id,
              );
            }

            for denied_command in &permission.commands.deny {
              resolve_command(
                &mut denied_commands,
                format!("plugin:{plugin_name}|{denied_command}"),
                capability,
                current_scope_id,
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
    allowed.scope.sort();

    let mut hasher = DefaultHasher::new();
    allowed.scope.hash(&mut hasher);
    let hash = hasher.finish() as usize;

    allowed.resolved_scope_key = hash;

    let resolved_scope = ResolvedScope {
      allow: allowed
        .scope
        .iter()
        .flat_map(|s| command_scopes.get(s).unwrap().allow.clone())
        .collect(),
      deny: allowed
        .scope
        .iter()
        .flat_map(|s| command_scopes.get(s).unwrap().deny.clone())
        .collect(),
    };

    resolved_scopes.insert(hash, resolved_scope);
  }

  let global_scope = ResolvedScope {
    allow: global_scope
      .iter_mut()
      .flat_map(|s| s.allow.take())
      .collect(),
    deny: global_scope
      .iter_mut()
      .flat_map(|s| s.deny.take())
      .collect(),
  };

  let resolved = Resolved {
    allowed_commands: allowed_commands
      .into_iter()
      .map(|(key, cmd)| {
        (
          key,
          ResolvedCommand {
            windows: cmd.windows,
            scope: cmd.resolved_scope_key,
          },
        )
      })
      .collect(),
    denied_commands: denied_commands
      .into_iter()
      .map(|(key, cmd)| {
        (
          key,
          ResolvedCommand {
            windows: cmd.windows,
            scope: cmd.resolved_scope_key,
          },
        )
      })
      .collect(),
    command_scope: resolved_scopes,
    global_scope,
  };

  Ok(resolved)
}

fn resolve_command(
  commands: &mut HashMap<CommandKey, ResolvedCommandTemp>,
  command: String,
  capability: &Capability,
  scope_id: usize,
) {
  let contexts = match &capability.context {
    CapabilityContext::Local => {
      vec![ExecutionContext::Local]
    }
    CapabilityContext::Remote { dangerous_remote } => dangerous_remote
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
    resolved.scope.push(scope_id);
  }
}

// get the permissions from a permission set
fn get_permission_set_permissions<'a>(
  manifest: &'a Manifest,
  set: &'a PermissionSet,
) -> Result<Vec<&'a Permission>, String> {
  let mut permissions = Vec::new();

  for p in &set.permissions {
    if let Some(permission) = manifest.permissions.get(p) {
      permissions.push(permission);
    } else if let Some(permission_set) = manifest.permission_sets.get(p) {
      permissions.extend(get_permission_set_permissions(manifest, permission_set)?);
    } else {
      return Err(format!(
        "permission {p} not found from set {}",
        set.identifier
      ));
    }
  }

  Ok(permissions)
}

fn get_permissions<'a>(
  plugin_name: &'a str,
  permission_name: &'a str,
  acl: &'a HashMap<String, Manifest>,
) -> Result<Vec<&'a Permission>, String> {
  let manifest = acl.get(plugin_name).ok_or_else(|| {
    format!(
      "unknown plugin {plugin_name}, expected one of {:?}",
      acl.keys().cloned().collect::<Vec<_>>().join(", ")
    )
  })?;

  if permission_name == "default" {
    Ok(vec![manifest.default_permission.as_ref().ok_or_else(
      || format!("plugin {plugin_name} has no default permission"),
    )?])
  } else if let Some(set) = manifest.permission_sets.get(permission_name) {
    get_permission_set_permissions(manifest, set)
  } else if let Some(permission) = manifest.permissions.get(permission_name) {
    Ok(vec![permission])
  } else {
    Err(format!(
      "unknown permission {permission_name} for plugin {plugin_name}"
    ))
  }
}
