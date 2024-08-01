// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{error::Error, path::PathBuf};

use schemars::schema_for;
use tauri_utils::acl::capability::Capability;
use tauri_utils::acl::{Permission, Scopes};
use tauri_utils::write_if_changed;

macro_rules! schema {
  ($name:literal, $path:ty) => {
    (concat!($name, "-schema.json"), schema_for!($path))
  };
}

pub fn main() -> Result<(), Box<dyn Error>> {
  let schemas = [
    schema!("capability", Capability),
    schema!("permission", Permission),
    schema!("scope", Scopes),
  ];

  let out = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
  for (filename, schema) in schemas {
    let schema = serde_json::to_string_pretty(&schema)?;
    write_if_changed(out.join(filename), schema)?;
  }

  Ok(())
}
