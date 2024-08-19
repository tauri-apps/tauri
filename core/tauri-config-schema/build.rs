// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{error::Error, path::PathBuf};
use tauri_utils::{config::Config, write_if_changed};

pub fn main() -> Result<(), Box<dyn Error>> {
  let schema = schemars::schema_for!(Config);
  let schema = serde_json::to_string_pretty(&schema)?;
  let out = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
  for path in ["schema.json", "../../tooling/cli/schema.json"] {
    write_if_changed(out.join(path), &schema)?;
  }

  Ok(())
}
