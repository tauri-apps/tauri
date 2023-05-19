use crate::{helpers::app_paths::tauri_dir, Result};

mod config;
mod manifest;

pub fn command() -> Result<()> {
  let tauri_dir = tauri_dir();

  config::migrate(&tauri_dir)?;
  manifest::migrate(&tauri_dir)?;

  Ok(())
}
