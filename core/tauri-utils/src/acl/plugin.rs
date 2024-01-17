// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plugin ACL types.

use std::{collections::HashMap, num::NonZeroU64};

use super::{Commands, Permission, PermissionSet, Scopes};
use serde::{Deserialize, Serialize};

/// The default permission of the plugin.
///
/// Works similarly to a permission with the "default" identifier.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DefaultPermission {
  /// The version of the permission.
  pub version: Option<NonZeroU64>,

  /// Human-readable description of what the permission does.
  pub description: Option<String>,

  /// Allowed or denied commands when using this permission.
  #[serde(default)]
  pub commands: Commands,

  /// Allowed or denied scoped when using this permission.
  #[serde(default)]
  pub scope: Scopes,
}

/// Permission file that can define a default permission, a set of permissions or a list of inlined permissions.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PermissionFile {
  /// The default permission set for the plugin
  pub default: Option<DefaultPermission>,

  /// A list of permissions sets defined
  #[serde(default)]
  pub set: Vec<PermissionSet>,

  /// Test something!!
  pub test: Option<PermissionSet>,

  /// A list of inlined permissions
  #[serde(default)]
  pub permission: Vec<Permission>,
}

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
