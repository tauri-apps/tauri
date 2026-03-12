// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{error::Error, path::PathBuf};

use serde::Deserialize;
use tauri_utils::{
  acl::{Permission, Scopes, capability::Capability},
  config::Config,
  write_if_changed,
};

macro_rules! schema {
  ($name:literal, $path:ty) => {
    (
      concat!($name, ".schema.json"),
      schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft07())
        .into_root_schema_for::<$path>(),
    )
  };
}

#[derive(Deserialize)]
pub struct VersionMetadata {
  tauri: String,
}

pub fn main() -> Result<(), Box<dyn Error>> {
  let schemas = [
    schema!("capability", Capability),
    schema!("permission", Permission),
    schema!("scope", Scopes),
  ];

  let out = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

  let schemas_dir = out.join("schemas");
  std::fs::create_dir_all(&schemas_dir)?;

  for (filename, schema) in schemas {
    let schema = serde_json::to_string_pretty(&schema)?;
    write_if_changed(schemas_dir.join(filename), &schema)?;
  }

  // write config schema file
  {
    let metadata = include_str!("../tauri-cli/metadata-v2.json");
    let tauri_ver = serde_json::from_str::<VersionMetadata>(metadata)?.tauri;

    // set $id for generated schema
    let (filename, mut config_schema) = schema!("config", Config);
    config_schema.insert(
      "$id".to_string(),
      format!("https://schema.tauri.app/config/{tauri_ver}").into(),
    );

    let config_schema = serde_json::to_string_pretty(&config_schema)?;
    write_if_changed(schemas_dir.join(filename), &config_schema)?;
    write_if_changed(out.join("../tauri-cli/config.schema.json"), config_schema)?;
  }

  Ok(())
}
