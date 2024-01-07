use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

pub use self::{identifier::*, value::*};

pub mod capability;
pub mod identifier;
pub mod plugin;
pub mod resolved;
pub mod value;

/// Allowed and denied commands inside a permission.
///
/// If two commands clash inside of `allow` and `deny`, it should be denied by default.
#[derive(Debug, Default, Deserialize)]
pub struct Commands {
  /// Allowed command.
  #[serde(default)]
  pub allow: Vec<String>,

  /// Denied command, which takes priority.
  #[serde(default)]
  pub deny: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Scopes {
  allow: Option<Value>,
  deny: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct InlinedPermission {
  /// The version of the permission.
  version: Option<NonZeroU64>,

  /// A unique identifier for the permission.
  identifier: Option<String>,

  /// Human-readable description of what the permission does.
  description: Option<String>,

  /// Allowed or denied commands when using this permission.
  #[serde(default)]
  commands: Commands,

  /// Allowed or denied scoped when using this permission.
  #[serde(default)]
  scopes: Scopes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionId {
  inner: String,
}

/// A permission.
#[derive(Debug, Deserialize)]
pub struct Permission {
  #[serde(flatten)]
  inner: InlinedPermission,
}

#[derive(Debug, Deserialize)]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  identifier: Identifier,

  /// Human-readable description of what the permission does.
  description: String,

  /// All permissions this set contains.
  permissions: Vec<Permission>,
}
