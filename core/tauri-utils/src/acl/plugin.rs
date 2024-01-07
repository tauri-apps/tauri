use std::collections::HashMap;

use super::{Permission, PermissionSet};
use serde::Serialize;

/// Plugin manifest.
#[derive(Serialize)]
pub struct Manifest {
  /// Default permission.
  pub default_permission: Option<Permission>,
  /// Plugin permissions.
  pub permissions: HashMap<String, Permission>,
  /// Plugin permission sets.
  pub permission_sets: HashMap<String, PermissionSet>,
}
