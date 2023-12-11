use std::num::NonZeroU64;

pub use self::{identifier::*, value::*};

mod identifier;
pub mod plugin;
mod value;

/// Allowed and denied commands inside a permission.
///
/// If two commands clash inside of `allow` and `deny`, it should be denied by default.
#[derive(Debug)]
pub struct Commands {
  /// Allowed command.
  pub allow: Vec<String>,

  /// Denied command, which takes priority.
  pub deny: Vec<String>,
}

#[derive(Debug)]
pub struct Scopes {
  allow: Value,
  deny: Value,
}

#[derive(Debug)]
pub struct InlinedPermission {
  /// The version of the permission.
  version: Option<NonZeroU64>,

  /// A unique identifier for the permission.
  identifier: Option<String>,

  /// Human-readable description of what the permission does.
  description: Option<String>,

  /// Allowed or denied commands when using this permission.
  commands: Commands,

  /// Allowed or denied scoped when using this permission.
  scopes: Scopes,
}

/// A permission.
#[derive(Debug)]
pub struct Permission {
  inner: InlinedPermission,
}

#[derive(Debug)]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  identifier: Identifier,

  /// Human-readable description of what the permission does.
  description: String,

  /// All permissions this set contains.
  permissions: Vec<Permission>,
}
