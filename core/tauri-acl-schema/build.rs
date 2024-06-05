// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  error::Error,
  fs::File,
  io::{BufWriter, Write},
  path::PathBuf,
};

use schemars::schema::RootSchema;

pub fn main() -> Result<(), Box<dyn Error>> {
  let cap_schema = schemars::schema_for!(tauri_utils::acl::capability::Capability);
  let perm_schema = schemars::schema_for!(tauri_utils::acl::Permission);
  let scope_schema = schemars::schema_for!(tauri_utils::acl::Scopes);

  let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

  write_schema_file(cap_schema, crate_dir.join("capability-schema.json"))?;
  write_schema_file(perm_schema, crate_dir.join("permission-schema.json"))?;
  write_schema_file(scope_schema, crate_dir.join("scope-schema.json"))?;

  Ok(())
}

fn write_schema_file(schema: RootSchema, outpath: PathBuf) -> Result<(), Box<dyn Error>> {
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let mut schema_file = BufWriter::new(File::create(outpath)?);
  write!(schema_file, "{schema_str}")?;

  Ok(())
}
