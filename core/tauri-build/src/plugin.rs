// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_utils::plugin::Capability;
pub use tauri_utils::plugin::{Manifest, ManifestMap, ScopeType};

use std::{
  collections::BTreeMap,
  env::{var_os, vars_os},
  fs::{read_to_string, write},
  path::PathBuf,
};

const PLUGIN_METADATA_KEY: &str = "PLUGIN_MANIFEST_PATH";

pub fn set_manifest(mut manifest: Manifest) {
  for feature in &manifest.features {
    let feature_capability_id = format!("allow-{feature}");
    if !manifest
      .capabilities
      .iter()
      .any(|c| c.id == feature_capability_id)
    {
      manifest.capabilities.push(Capability {
        id: feature_capability_id,
        component: None,
        description: Some(format!("Allows the {feature} functionality")),
        features: vec![feature.clone()],
        scope: Default::default(),
      });
    }
  }

  let manifest_str = serde_json::to_string(&manifest).expect("failed to serialize plugin manifest");
  let manifest_path = var_os("OUT_DIR")
    .map(PathBuf::from)
    .expect(
      "missing OUT_DIR environment variable.. are you sure you are running this on a build script?",
    )
    .join(format!("{}-plugin-manifest.json", manifest.plugin));
  write(&manifest_path, manifest_str).expect("failed to save manifest file");

  println!(
    "cargo:{}_{PLUGIN_METADATA_KEY}={}",
    manifest.plugin,
    manifest_path.display()
  );
}

pub(crate) fn manifests() -> ManifestMap {
  let mut manifests = BTreeMap::new();

  for (key, value) in vars_os() {
    let key = key.to_string_lossy();
    if let Some(_plugin_crate_name) = key
      .strip_prefix("DEP_")
      .and_then(|v| v.strip_suffix(&format!("_{PLUGIN_METADATA_KEY}")))
    {
      let plugin_manifest_path = PathBuf::from(value);
      let plugin_manifest_str =
        read_to_string(&plugin_manifest_path).expect("failed to read plugin manifest");
      let manifest: Manifest =
        serde_json::from_str(&plugin_manifest_str).expect("failed to deserialize plugin manifest");
      manifests.insert(manifest.plugin.clone(), manifest);
    }
  }

  manifests.into()
}
