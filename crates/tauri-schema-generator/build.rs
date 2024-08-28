// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{error::Error, path::PathBuf};

use tauri_utils::{
  acl::{capability::Capability, Permission, Scopes},
  config::Config,
  write_if_changed,
};

macro_rules! schema {
  ($name:literal, $path:ty) => {
    (concat!($name, ".schema.json"), schemars::schema_for!($path))
  };
}

pub fn main() -> Result<(), Box<dyn Error>> {
  let schemas = [
    schema!("config", Config),
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

    if filename.starts_with("config") {
      write_if_changed(out.join("../tauri-cli/config.schema.json"), schema)?;
    }
  }

  Ok(())
}
