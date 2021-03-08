use std::{
  env,
  env::current_dir,
  error::Error,
  fs::{read_to_string, File},
  io::{BufWriter, Write},
  path::Path,
};

mod config_definition;

pub fn main() -> Result<(), Box<dyn Error>> {
  let out_dir = env::var("OUT_DIR")?;

  let dest_bundle_umd_path = Path::new(&out_dir).join("tauri.bundle.umd.js");
  let mut bundle_umd_file = BufWriter::new(File::create(&dest_bundle_umd_path)?);

  let bundle_umd_path = current_dir()?.join("../../api/dist/tauri.bundle.umd.js");
  println!("cargo:rerun-if-changed={:?}", bundle_umd_path);
  if let Ok(bundle_umd_js) = read_to_string(bundle_umd_path) {
    write!(bundle_umd_file, "{}", bundle_umd_js)?;
  } else {
    write!(
      bundle_umd_file,
      r#"throw new Error("you are trying to use the global Tauri script but the @tauri-apps/api package wasn't compiled; run `yarn build` first")"#
    )?;
  }

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
