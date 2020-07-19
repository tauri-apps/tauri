use super::{app_paths::tauri_dir, config::get as get_config};

use convert_case::{Case, Casing};
use toml_edit::{Array, Document, Item, Value};

use std::fs::File;
use std::io::{Read, Write};

pub fn rewrite_manifest() -> crate::Result<()> {
  let config = get_config()?;
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest_str = String::new();
  let mut manifest_file = File::open(&manifest_path)?;
  manifest_file.read_to_string(&mut manifest_str)?;
  let mut manifest: Document = manifest_str.parse::<Document>()?;
  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

  let entry = dependencies.entry("tauri");
  let tauri = entry.as_value_mut();
  if let Some(tauri) = tauri {
    let mut features: Array = Default::default();

    let allowlist = &config.tauri.allowlist;
    if *allowlist.get("all").unwrap_or(&false) {
      features.push("all-api".to_string()).unwrap();
    } else {
      for (feature, enabled) in allowlist.iter() {
        if *enabled {
          features.push(feature.to_case(Case::Kebab)).unwrap();
        }
      }
    }

    if config.tauri.cli.is_some() {
      features.push("cli".to_string()).unwrap();
    }

    match tauri {
      Value::InlineTable(tauri_def) => {
        let manifest_features =
          tauri_def.get_or_insert("features", Value::Array(Default::default()));
        *manifest_features = Value::Array(features);
      }
      _ => {
        return Err(anyhow::anyhow!(
          "Unsupported tauri dependency format on Cargo.toml"
        ))
      }
    }

    let mut manifest_file = File::create(&manifest_path)?;
    manifest_file.write_all(
      manifest
        .to_string_in_original_order()
        .replace(r#"" ,features =["#, r#"", features = ["#)
        .as_bytes(),
    )?;
    manifest_file.flush()?;
  }

  Ok(())
}
