use crate::acl::PermissionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityId {
  inner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CapabilitySet {
  inner: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
  identifier: CapabilityId,
  description: String,
  #[serde(default)]
  context: CapabilityContext,
  windows: Vec<String>,
  permissions: Vec<PermissionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityContext {
  Local,
  Remote { dangerous_remote: Vec<String> },
}

impl Default for CapabilityContext {
  fn default() -> Self {
    Self::Local
  }
}
