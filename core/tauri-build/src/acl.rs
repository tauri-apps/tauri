use std::collections::HashMap;

use anyhow::{Context, Result};
use tauri_utils::acl::{plugin::Manifest, InlinedPermission, Permission, PermissionSet};

pub fn process() -> Result<HashMap<String, Manifest>> {
  let permission_map =
    tauri_plugin::acl::read_permissions().context("failed to read plugin permissions")?;

  let mut processed = HashMap::new();
  for (plugin_name, permission_files) in permission_map {
    let mut manifest = Manifest {
      default_permission: None,
      permissions: HashMap::new(),
      permission_sets: HashMap::new(),
    };

    for permission_file in permission_files {
      if let Some(default) = permission_file.default {
        manifest.default_permission.replace(Permission {
          inner: InlinedPermission {
            identifier: "default".into(),
            version: default.version,
            description: default.description,
            commands: default.commands,
            scopes: default.scopes,
          },
        });
      }

      if let Some(permissions) = permission_file.permission {
        manifest.permissions.extend(
          permissions
            .into_iter()
            .map(|p| (p.inner.identifier.clone(), p))
            .collect::<HashMap<_, _>>(),
        );
      }
      if let Some(sets) = permission_file.set {
        manifest.permission_sets.extend(
          sets
            .into_iter()
            .map(|set| {
              (
                set.identifier.clone(),
                PermissionSet {
                  identifier: set.identifier,
                  description: set.description,
                  permissions: set.permissions,
                },
              )
            })
            .collect::<HashMap<_, _>>(),
        );
      }
    }

    processed.insert(plugin_name, manifest);
  }

  Ok(processed)
}
