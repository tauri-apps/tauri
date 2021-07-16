// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

use std::{
  env::current_dir,
  error::Error,
  fs::File,
  io::{BufWriter, Write},
};

mod config_definition;

pub fn main() -> Result<(), Box<dyn Error>> {
  let schema = schemars::schema_for!(config_definition::Config);
  let schema_file_path = current_dir()?.join("schema.json");
  let mut schema_file = BufWriter::new(File::create(&schema_file_path)?);
  write!(
    schema_file,
    "{}",
    serde_json::to_string_pretty(&schema).unwrap()
  )?;

  Ok(())
}
