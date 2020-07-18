use std::env::current_dir;
use std::error::Error;
use std::process::Command;

pub fn main() -> Result<(), Box<dyn Error>> {
  if !current_dir()?
    .join("tauri.js/api/tauri.bundle.umd.js")
    .exists()
  {
    println!("cargo:rerun-if-changed=./tauri.js/api-src");
    let exit_status = Command::new("yarn").current_dir("./tauri.js").status()?;
    if !exit_status.success() {
      panic!("Failed to install yarn deps");
    }

    let exit_status = Command::new("yarn")
      .arg("build:api")
      .current_dir("./tauri.js")
      .status()?;
    if !exit_status.success() {
      panic!("Failed to build api");
    }
  }

  Ok(())
}
