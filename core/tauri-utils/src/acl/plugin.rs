// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plugin ACL types.

use std::collections::HashMap;

use super::{build::PermissionFile, Permission, PermissionSet};
use serde::{Deserialize, Serialize};

/// Plugin manifest.
#[derive(Serialize, Deserialize)]
pub struct Manifest {
  /// Default permission.
  pub default_permission: Option<Permission>,
  /// Plugin permissions.
  pub permissions: HashMap<String, Permission>,
  /// Plugin permission sets.
  pub permission_sets: HashMap<String, PermissionSet>,
}

impl Manifest {
  /// Creates a new manifest from a list of permission files.
  pub fn from_files(permission_files: Vec<PermissionFile>) -> Self {
    let mut manifest = Self {
      default_permission: None,
      permissions: HashMap::new(),
      permission_sets: HashMap::new(),
    };

    for permission_file in permission_files {
      if let Some(default) = permission_file.default {
        manifest.default_permission.replace(Permission {
          identifier: "default".into(),
          version: default.version,
          description: default.description,
          commands: default.commands,
          scope: default.scope,
        });
      }

      manifest.permissions.extend(
        permission_file
          .permission
          .into_iter()
          .map(|p| (p.identifier.clone(), p))
          .collect::<HashMap<_, _>>(),
      );

      manifest.permission_sets.extend(
        permission_file
          .set
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

    manifest
  }
}
