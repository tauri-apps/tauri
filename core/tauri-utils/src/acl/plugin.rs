// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plugin ACL types.

use std::{collections::BTreeMap, num::NonZeroU64};

use super::{Permission, PermissionSet};
use serde::{Deserialize, Serialize};

/// The default permission set of the plugin.
///
/// Works similarly to a permission with the "default" identifier.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DefaultPermission {
  /// The version of the permission.
  pub version: Option<NonZeroU64>,

  /// Human-readable description of what the permission does.
  pub description: Option<String>,

  /// All permissions this set contains.
  pub permissions: Vec<String>,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
  /// Default permission.
  pub default_permission: Option<PermissionSet>,
  /// Plugin permissions.
  pub permissions: BTreeMap<String, Permission>,
  /// Plugin permission sets.
  pub permission_sets: BTreeMap<String, PermissionSet>,
}

impl Manifest {
  /// Creates a new manifest from a list of permission files.
  pub fn from_files(permission_files: Vec<PermissionFile>) -> Self {
    let mut manifest = Self {
      default_permission: None,
      permissions: BTreeMap::new(),
      permission_sets: BTreeMap::new(),
    };

    for permission_file in permission_files {
      if let Some(default) = permission_file.default {
        manifest.default_permission.replace(PermissionSet {
          identifier: "default".into(),
          description: default
            .description
            .unwrap_or_else(|| "Default plugin permissions.".to_string()),
          permissions: default.permissions,
        });
      }

      manifest.permissions.extend(
        permission_file
          .permission
          .into_iter()
          .map(|p| (p.identifier.clone(), p))
          .collect::<BTreeMap<_, _>>(),
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
          .collect::<BTreeMap<_, _>>(),
      );
    }

    manifest
  }
}
