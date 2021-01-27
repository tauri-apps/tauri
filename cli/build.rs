use std::env::current_dir;
use std::error::Error;
use std::process::Command;

pub fn main() -> Result<(), Box<dyn Error>> {
  if !current_dir()?
    .join("api-definitions/dist/tauri.bundle.umd.js")
    .exists()
  {
    println!("cargo:rerun-if-changed=./api-definition");
    let exit_status = Command::new("yarn")
      .current_dir("./api-definitions")
      .status()?;
    if !exit_status.success() {
      panic!("Failed to install @tauri-apps/api yarn deps");
    }

    let exit_status = Command::new("yarn")
      .arg("build")
      .current_dir("./api-definitions")
      .status()?;
    if !exit_status.success() {
      panic!("Failed to build @tauri-apps/api");
    }
  }

  Ok(())
}
