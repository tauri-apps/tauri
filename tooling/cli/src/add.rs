// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;
use colored::Colorize;
use regex::Regex;

use crate::{
  acl,
  helpers::{
    app_paths::{app_dir, tauri_dir},
    cargo,
    npm::PackageManager,
  },
  Result,
};

use std::{collections::HashMap, process::Command};

#[derive(Default)]
struct PluginMetadata {
  desktop_only: bool,
  rust_only: bool,
  builder: bool,
}

// known plugins with particular cases
fn plugins() -> HashMap<&'static str, PluginMetadata> {
  let mut plugins: HashMap<&'static str, PluginMetadata> = HashMap::new();

  // desktop-only
  for p in [
    "authenticator",
    "cli",
    "global-shortcut",
    "updater",
    "window-state",
  ] {
    plugins.entry(p).or_default().desktop_only = true;
  }

  // uses builder pattern
  for p in [
    "global-shortcut",
    "localhost",
    "log",
    "sql",
    "store",
    "stronghold",
    "updater",
    "window-state",
  ] {
    plugins.entry(p).or_default().builder = true;
  }

  // rust-only
  #[allow(clippy::single_element_loop)]
  for p in ["localhost"] {
    plugins.entry(p).or_default().rust_only = true;
  }

  plugins
}

#[derive(Debug, Parser)]
#[clap(about = "Add a tauri plugin to the project")]
pub struct Options {
  /// The plugin to add.
  pub plugin: String,
  /// Git tag to use.
  #[clap(short, long)]
  pub tag: Option<String>,
  /// Git rev to use.
  #[clap(short, long)]
  pub rev: Option<String>,
  /// Git branch to use.
  #[clap(short, long)]
  pub branch: Option<String>,
}

pub fn command(options: Options) -> Result<()> {
  let plugin = options.plugin;
  let plugin_snake_case = plugin.replace('-', "_");
  let crate_name = format!("tauri-plugin-{plugin}");
  let npm_name = format!("@tauri-apps/plugin-{plugin}");

  let mut plugins = plugins();
  let metadata = plugins.remove(plugin.as_str()).unwrap_or_default();

  let tauri_dir = tauri_dir();

  cargo::install_one(cargo::CargoInstallOptions {
    name: &crate_name,
    branch: options.branch.as_deref(),
    rev: options.rev.as_deref(),
    tag: options.tag.as_deref(),
    cwd: Some(&tauri_dir),
    target: metadata
      .desktop_only
      .then_some(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#),
  })?;

  if !metadata.rust_only {
    if let Some(manager) = std::panic::catch_unwind(app_dir)
      .map(Some)
      .unwrap_or_default()
      .map(PackageManager::from_project)
      .and_then(|managers| managers.into_iter().next())
    {
      let npm_spec = match (options.tag, options.rev, options.branch) {
        (Some(tag), None, None) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{tag}")
        }
        (None, Some(rev), None) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{rev}")
        }
        (None, None, Some(branch)) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{branch}")
        }
        (None, None, None) => npm_name,
        _ => anyhow::bail!("Only one of --tag, --rev and --branch can be specified"),
      };
      manager.install(&[npm_spec])?;
    }
  }

  let _ = acl::permission::add::command(acl::permission::add::Options {
    identifier: format!("{plugin}:default"),
    capability: None,
  });

  // add plugin init code to main.rs or lib.rs
  let plugin_init_fn = if plugin == "stronghold" {
    "Builder::new(|pass| todo!()).build()"
  } else if metadata.builder {
    "Builder::new().build()"
  } else {
    "init()"
  };
  let plugin_init = format!(".plugin(tauri_plugin_{plugin_snake_case}::{plugin_init_fn})");

  let re = Regex::new(r"(tauri\s*::\s*Builder\s*::\s*default\(\))(\s*)")?;
  for file in [tauri_dir.join("src/main.rs"), tauri_dir.join("src/lib.rs")] {
    let contents = std::fs::read_to_string(&file)?;

    if contents.contains(&plugin_init) {
      log::info!(
        "Plugin initialization code already found on {}",
        file.display()
      );
      return Ok(());
    }

    if re.is_match(&contents) {
      let out = re.replace(&contents, format!("$1$2{plugin_init}$2"));

      log::info!("Adding plugin to {}", file.display());
      std::fs::write(file, out.as_bytes())?;

      // run cargo fmt
      log::info!("Running `cargo fmt`...");
      let _ = Command::new("cargo")
        .arg("fmt")
        .current_dir(&tauri_dir)
        .status();
      return Ok(());
    }
  }

  let builder_code = if metadata.builder {
    format!(r#"+    .plugin(tauri_plugin_{plugin_snake_case}::Builder::new().build())"#,)
  } else {
    format!(r#"+    .plugin(tauri_plugin_{plugin_snake_case}::init())"#)
  };

  let rust_code = format!(
    r#" {}
{}
     {}"#,
    "tauri::Builder::default()".dimmed(),
    builder_code.normal().green(),
    r#".invoke_handler(tauri::generate_handler![])
     .run(tauri::generate_context!())
     .expect("error while running tauri application");"#
      .dimmed(),
  );

  log::warn!(
    "Couldn't find `{}` in `{}` or `{}`, you must enable the plugin in your Rust code manually:\n\n{}",
    "tauri::Builder".cyan(),
    "main.rs".cyan(),
    "lib.rs".cyan(),
    rust_code
  );

  Ok(())
}
