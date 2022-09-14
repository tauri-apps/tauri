// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::PathBuf,
  error::Error,
  fs::File,
  io::{BufWriter, Write},
};

pub fn main() -> Result<(), Box<dyn Error>> {
  let schema = schemars::schema_for!(tauri_utils::config::Config);
  let schema_file_path = PathBuf::from( std::env::var("CARGO_MANIFEST_DIR")?).join("schema.json");
  let mut schema_file = BufWriter::new(File::create(&schema_file_path)?);
  write!(
    schema_file,
    "{}",
    serde_json::to_string_pretty(&schema).unwrap()
  )?;

  Ok(())
}
