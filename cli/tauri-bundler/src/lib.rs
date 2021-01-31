pub mod bundle;
mod error;
pub use error::{Error, Result};

use bundle::Settings;
use std::process;

// Runs `cargo build` to make sure the binary file is up-to-date.
pub fn build_project(settings: &Settings) -> crate::Result<()> {
  let mut args = vec!["build".to_string()];

  if let Some(triple) = settings.target_triple() {
    args.push(format!("--target={}", triple));
  }

  if settings.is_release_build() {
    args.push("--release".to_string());
  }

  if let Some(features) = settings.build_features() {
    args.push(format!("--features={}", features.join(" ")));
  }

  let status = process::Command::new("cargo").args(args).status()?;
  if !status.success() {
    return Err(crate::Error::GenericError(format!(
      "Result of `cargo build` operation was unsuccessful: {}",
      status
    )));
  }
  Ok(())
}
