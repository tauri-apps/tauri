//! End-user abstraction for selecting permissions a window has access to.

use crate::acl::PermissionId;
use serde::{Deserialize, Serialize};

/// Identifier of a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityId {
  inner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

/// A set of direct capabilities grouped together under a new name.
pub struct CapabilitySet {
  inner: Vec<Capability>,
}

/// a grouping and boundary mechanism developers can use to separate windows or plugins functionality from each other at runtime.
///
/// If a window is not matching any capability then it has no access to the IPC layer at all.
///
/// This can be done to create trust groups and reduce impact of vulnerabilities in certain plugins or windows.
/// Windows can be added to a capability by exact name or glob patterns like *, admin-* or main-window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
  identifier: CapabilityId,
  description: String,
  #[serde(default)]
  context: CapabilityContext,
  windows: Vec<String>,
  permissions: Vec<PermissionId>,
}

/// Context of the capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityContext {
  /// Capability refers to local URL usage.
  Local,
  /// Capability refers to remote usage.
  Remote {
    /// Remote domain this capability refers to.
    dangerous_remote: Vec<String>,
  },
}

impl Default for CapabilityContext {
  fn default() -> Self {
    Self::Local
  }
}
