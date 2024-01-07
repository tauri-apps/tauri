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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Commands {
  /// Allowed command.
  #[serde(default)]
  pub allow: Vec<String>,

  /// Denied command, which takes priority.
  #[serde(default)]
  pub deny: Vec<String>,
}

/// A restriction of the command/endpoint functionality.
///
/// It can be of any serde serializable type and is used for allowing or preventing certain actions inside a Tauri command.
///
/// The scope is passed to the command and handled/enforced by the command itself.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scopes {
  /// Data that defines what is allowed by the scope.
  pub allow: Option<Value>,
  /// Data that defines what is denied by the scope.
  pub deny: Option<Value>,
}

/// Descriptions of explicit privileges of commands.
///
/// It can enable commands to be accessible in the frontend of the application.
///
/// If the scope is defined it can be used to fine grain control the access of individual or multiple commands.
#[derive(Debug, Serialize, Deserialize)]
pub struct InlinedPermission {
  /// The version of the permission.
  pub version: Option<NonZeroU64>,

  /// A unique identifier for the permission.
  pub identifier: String,

  /// Human-readable description of what the permission does.
  pub description: Option<String>,

  /// Allowed or denied commands when using this permission.
  #[serde(default)]
  pub commands: Commands,

  /// Allowed or denied scoped when using this permission.
  #[serde(default)]
  pub scopes: Scopes,
}

/// Identifier of a permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionId {
  inner: String,
}

/// A permission.
#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
  /// Permission data.
  #[serde(flatten)]
  pub inner: InlinedPermission,
}

/// A set of direct permissions grouped together under a new name.
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  pub identifier: String,

  /// Human-readable description of what the permission does.
  pub description: String,

  /// All permissions this set contains.
  pub permissions: Vec<String>,
}
