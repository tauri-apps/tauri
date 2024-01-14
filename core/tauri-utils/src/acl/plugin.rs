// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plugin ACL types.

use std::collections::HashMap;

use super::{Permission, PermissionSet};
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
