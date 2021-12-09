// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use anyhow::Result;

mod build;
mod dev;
mod helpers;
mod info;
mod init;
mod interface;
mod plugin;
mod signer;

use clap::{crate_version, load_yaml, App, AppSettings};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VersionMetadata {
  tauri: String,
  #[serde(rename = "tauri-build")]
  tauri_build: String,
}

fn main() -> Result<()> {
  let yaml = load_yaml!("cli.yml");
  let app = App::from(yaml)
    .version(crate_version!())
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::PropagateVersion)
    .setting(AppSettings::SubcommandRequired)
    .arg(clap::Arg::new("cargo").hidden(true).possible_value("tauri"));

  let matches = app.get_matches();

  if let Some(matches) = matches.subcommand_matches("dev") {
    dev::command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("build") {
    build::command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("signer") {
    signer::command(matches)?;
  } else if let Some(_) = matches.subcommand_matches("info") {
    info::command()?;
  } else if let Some(matches) = matches.subcommand_matches("init") {
    init::command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("plugin") {
    plugin::command(matches)?;
  }

  Ok(())
}
