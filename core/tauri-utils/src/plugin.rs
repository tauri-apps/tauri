// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plugin manifest types.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

const DEFAULT_CAPABILITY_ID: &str = "default";

/// Scope type definition.
#[derive(Debug, Serialize, Deserialize)]
pub enum ScopeType {
  /// String type.
  String,
}

/// Scope of a given capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityScope {
  /// Explicitly allow something.
  #[serde(default)]
  pub allowed: Vec<serde_json::Value>,
  /// Explicitly deny something. Takes precedence over [`Self::allowed`].
  #[serde(default)]
  pub blocked: Vec<serde_json::Value>,
}

/// A capability defines a set of features and scope enabled for the plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capability {
  /// The identifier of the capability. Must be unique.
  #[serde(default)]
  pub id: String,
  /// Describes the capability in a human readable format.
  pub description: String,
  /// List of features enabled by this capability.
  #[serde(default)]
  pub features: Vec<String>,
  /// Scope defined by this capability. Only applies to the given features.
  #[serde(default)]
  pub scope: CapabilityScope,
}

/// Plugin manifest.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
  /// Plugin name.
  pub plugin: String,
  /// Default capability.
  pub default_capability: Option<Capability>,
  /// List of capabilities defined by the plugin.
  pub capabilities: Vec<Capability>,
  /// List of features defined by the plugin.
  pub features: Vec<String>,
  /// Scope types.
  pub scope_type: Vec<ScopeType>,
}

impl Manifest {
  /// Creates a new empty plugin manifest.
  pub fn new(plugin: impl Into<String>) -> Self {
    Self {
      plugin: plugin.into(),
      default_capability: None,
      capabilities: Vec::new(),
      features: Vec::new(),
      scope_type: Vec::new(),
    }
  }

  /// Sets the plugin's default capability set.
  pub fn default_capability(mut self, default_capability: impl AsRef<str>) -> Self {
    let mut capability: Capability = serde_json::from_str(default_capability.as_ref())
      .expect("failed to deserialize default capability");
    assert!(
      capability.id.is_empty(),
      "default capability cannot have an specific identifier"
    );
    capability.id = DEFAULT_CAPABILITY_ID.into();
    self.default_capability.replace(capability);
    self
  }

  /// Appends a capability JSON. See [`Capability`].
  pub fn capability(mut self, capability: impl AsRef<str>) -> Self {
    let capability: Capability =
      serde_json::from_str(capability.as_ref()).expect("failed to deserialize default capability");
    assert!(
      !capability.id.is_empty(),
      "capability must have an specific identifier"
    );
    self.capabilities.push(capability);
    self
  }

  /// Appends the given feature on the list of plugin's features.
  pub fn feature(mut self, feature: impl Into<String>) -> Self {
    self.features.push(feature.into());
    self
  }

  /// Appends the given list of features.
  pub fn features<I: IntoIterator<Item = S>, S: Into<String>>(mut self, features: I) -> Self {
    for feature in features {
      self = self.feature(feature);
    }
    self
  }

  /// Appends the given scope type.
  pub fn scope_type(mut self, ty: ScopeType) -> Self {
    self.scope_type.push(ty);
    self
  }
}

/// A collection mapping a plugin name to its manifest.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ManifestMap(HashMap<String, Manifest>);

impl From<HashMap<String, Manifest>> for ManifestMap {
  fn from(value: HashMap<String, Manifest>) -> Self {
    Self(value)
  }
}

impl ManifestMap {
  /// Finds the capability with the given identifier.
  pub fn find_capability(&self, id: &str) -> Option<(String, Capability)> {
    for (plugin, manifest) in &self.0 {
      if id == format!("{DEFAULT_CAPABILITY_ID}-{plugin}") {
        return Some((
          plugin.clone(),
          manifest.default_capability.clone().unwrap_or_default(),
        ));
      }
      for capability in &manifest.capabilities {
        if capability.id == id {
          return Some((plugin.clone(), capability.clone()));
        }
      }
    }

    None
  }
}
