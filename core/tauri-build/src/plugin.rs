use serde::{Deserialize, Serialize};

use std::{
  collections::HashMap,
  env::{var_os, vars_os},
  fs::{read_to_string, write},
  path::PathBuf,
};

const PLUGIN_METADATA_KEY: &str = "PLUGIN_MANIFEST_PATH";
const DEFAULT_CAPABILITY_ID: &str = "default";

#[derive(Debug, Serialize, Deserialize)]
pub enum ScopeType {
  String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityScope {
  #[serde(default)]
  allowed: Vec<serde_json::Value>,
  #[serde(default)]
  blocked: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capability {
  #[serde(default)]
  pub(crate) id: String,
  pub(crate) description: String,
  #[serde(default)]
  pub(crate) features: Vec<String>,
  #[serde(default)]
  pub(crate) scope: CapabilityScope,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
  plugin: String,
  default_capability: Option<Capability>,
  capabilities: Vec<Capability>,
  features: Vec<String>,
  scope_type: Vec<ScopeType>,
}

impl Manifest {
  pub fn new(plugin: impl Into<String>) -> Self {
    Self {
      plugin: plugin.into(),
      default_capability: None,
      capabilities: Vec::new(),
      features: Vec::new(),
      scope_type: Vec::new(),
    }
  }

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

  pub fn feature(mut self, feature: impl Into<String>) -> Self {
    self.features.push(feature.into());
    self
  }

  pub fn features<I: IntoIterator<Item = S>, S: Into<String>>(mut self, features: I) -> Self {
    for feature in features {
      self = self.feature(feature);
    }
    self
  }

  pub fn scope_type(mut self, ty: ScopeType) -> Self {
    self.scope_type.push(ty);
    self
  }
}

pub fn set_manifest(manifest: Manifest) {
  let manifest_str = serde_json::to_string(&manifest).expect("failed to serialize plugin manifest");
  let manifest_path = var_os("OUT_DIR")
    .map(PathBuf::from)
    .expect(
      "missing OUT_DIR environment variable.. are you sure you are running this on a build script?",
    )
    .join("plugin-manifest.json");
  write(&manifest_path, manifest_str).expect("failed to save manifest file");

  println!("cargo:{PLUGIN_METADATA_KEY}={}", manifest_path.display());
}

#[derive(Serialize)]
pub(crate) struct ManifestMap(HashMap<String, Manifest>);

impl ManifestMap {
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

pub(crate) fn manifests() -> ManifestMap {
  let mut manifests = HashMap::new();

  for (key, value) in vars_os() {
    let key = key.to_string_lossy();
    if let Some(_plugin_crate_name) = key
      .strip_prefix("DEP_")
      .and_then(|v| v.strip_suffix(&format!("_{PLUGIN_METADATA_KEY}")))
      .map(|p| p.to_lowercase())
    {
      let plugin_manifest_path = PathBuf::from(value);
      let plugin_manifest_str =
        read_to_string(&plugin_manifest_path).expect("failed to read plugin manifest");
      let manifest: Manifest =
        serde_json::from_str(&plugin_manifest_str).expect("failed to deserialize plugin manifest");

      manifests.insert(manifest.plugin.clone(), manifest);
    }
  }

  ManifestMap(manifests)
}
