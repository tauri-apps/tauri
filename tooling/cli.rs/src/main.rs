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

use clap::{AppSettings, FromArgMatches, IntoApp, Parser, Subcommand};

#[derive(serde::Deserialize)]
pub struct VersionMetadata {
  tauri: String,
  #[serde(rename = "tauri-build")]
  tauri_build: String,
}

#[derive(Parser)]
#[clap(author, version, about, bin_name("cargo tauri"))]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Build(build::Options),
  Dev(dev::Options),
  Info(info::Options),
  Init(init::Options),
  Plugin(plugin::Cli),
  Signer(signer::Cli),
}

fn format_error<I: IntoApp>(err: clap::Error) -> clap::Error {
  let mut app = I::into_app();
  err.format(&mut app)
}

fn main() -> Result<()> {
  let matches = <Cli as IntoApp>::into_app()
    .arg(clap::Arg::new("cargo").hide(true).possible_value("tauri"))
    .get_matches();
  let res = <Cli as FromArgMatches>::from_arg_matches(&matches).map_err(format_error::<Cli>);
  let cli = match res {
    Ok(s) => s,
    Err(e) => e.exit(),
  };

  match cli.command {
    Commands::Build(options) => build::command(options)?,
    Commands::Dev(options) => dev::command(options)?,
    Commands::Info(options) => info::command(options)?,
    Commands::Init(options) => init::command(options)?,
    Commands::Plugin(cli) => plugin::command(cli)?,
    Commands::Signer(cli) => signer::command(cli)?,
  }

  /*if let Some(matches) = matches.subcommand_matches("dev") {
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
  }*/

  Ok(())
}
