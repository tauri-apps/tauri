// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::PluginIosFramework;
use crate::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(about = "Initializes a new Tauri plugin project")]
pub struct Options {
  /// Name of your Tauri plugin
  plugin_name: String,
  /// Initializes a Tauri plugin without the TypeScript API
  #[clap(long)]
  no_api: bool,
  /// Initialize without an example project.
  #[clap(long)]
  no_example: bool,
  /// Set target directory for init
  #[clap(short, long)]
  directory: Option<String>,
  /// Author name
  #[clap(short, long)]
  author: Option<String>,
  /// Whether to initialize an Android project for the plugin.
  #[clap(long)]
  android: bool,
  /// Whether to initialize an iOS project for the plugin.
  #[clap(long)]
  ios: bool,
  /// Whether to initialize Android and iOS projects for the plugin.
  #[clap(long)]
  mobile: bool,
  /// Type of framework to use for the iOS project.
  #[clap(long)]
  #[clap(default_value_t = PluginIosFramework::default())]
  pub(crate) ios_framework: PluginIosFramework,
  /// Generate github workflows
  #[clap(long)]
  github_workflows: bool,

  /// Initializes a Tauri core plugin (internal usage)
  #[clap(long, hide(true))]
  tauri: bool,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  tauri_path: Option<PathBuf>,
}

impl From<Options> for super::init::Options {
  fn from(o: Options) -> Self {
    Self {
      plugin_name: Some(o.plugin_name),
      no_api: o.no_api,
      no_example: o.no_example,
      directory: o.directory.unwrap(),
      author: o.author,
      android: o.android,
      ios: o.ios,
      mobile: o.mobile,
      ios_framework: o.ios_framework,
      github_workflows: o.github_workflows,

      tauri: o.tauri,
      tauri_path: o.tauri_path,
    }
  }
}

pub fn command(mut options: Options) -> Result<()> {
  let cwd = std::env::current_dir()?;
  if let Some(dir) = &options.directory {
    std::fs::create_dir_all(cwd.join(dir))?;
  } else {
    let target = cwd.join(format!("tauri-plugin-{}", options.plugin_name));
    std::fs::create_dir_all(&target)?;
    options.directory.replace(target.display().to_string());
  }

  super::init::command(options.into())
}
