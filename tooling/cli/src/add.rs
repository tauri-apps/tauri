// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
use clap::Parser;

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    cross_command,
    npm::PackageManager,
  },
  Result,
};

use std::{collections::HashMap, process::Command};

#[derive(Debug, Parser)]
#[clap(about = "Add a tauri plugin to the project")]
pub struct Options {
  /// The plugin to add.
  plugin: String,
  /// Git tag to use.
  #[clap(short, long)]
  tag: Option<String>,
  /// Git rev to use.
  #[clap(short, long)]
  rev: Option<String>,
  /// Git branch to use.
  #[clap(short, long)]
  branch: Option<String>,
}

pub fn command(options: Options) -> Result<()> {
  let plugin = options.plugin;
  let plugin_snake_case = plugin.replace('-', "_");
  let crate_name = format!("tauri-plugin-{plugin}");
  let npm_name = format!("@tauri-apps/plugin-{plugin}");

  let mut plugins = plugins();
  let metadata = plugins.remove(plugin.as_str()).unwrap_or_default();

  let mut cargo = Command::new("cargo");
  cargo.current_dir(tauri_dir()).arg("add").arg(&crate_name);

  if options.tag.is_some() || options.rev.is_some() || options.branch.is_some() {
    cargo
      .arg("--git")
      .arg("https://github.com/tauri-apps/plugins-workspace");
  }

  if metadata.desktop_only {
    cargo
      .arg("--target")
      .arg(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#);
  }

  let npm_spec = match (options.tag, options.rev, options.branch) {
    (Some(tag), None, None) => {
      cargo.args(["--tag", &tag]);
      format!("tauri-apps/tauri-plugin-{plugin}#{tag}")
    }
    (None, Some(rev), None) => {
      cargo.args(["--rev", &rev]);
      format!("tauri-apps/tauri-plugin-{plugin}#{rev}")
    }
    (None, None, Some(branch)) => {
      cargo.args(["--branch", &branch]);
      format!("tauri-apps/tauri-plugin-{plugin}#{branch}")
    }
    (None, None, None) => npm_name,
    _ => anyhow::bail!("Only one of --tag, --rev and --branch can be specified"),
  };

  log::info!("Installing Cargo dependency {crate_name}...");
  let status = cargo.status().context("failed to run `cargo add`")?;
  if !status.success() {
    anyhow::bail!("Failed to install Cargo dependency");
  }

  if !metadata.rust_only {
    if let Some(manager) = std::panic::catch_unwind(app_dir)
      .map(Some)
      .unwrap_or_default()
      .map(PackageManager::from_project)
      .and_then(|managers| managers.into_iter().next())
    {
      let mut cmd = match manager {
        PackageManager::Npm => cross_command("npm"),
        PackageManager::Pnpm => cross_command("pnpm"),
        PackageManager::Yarn => cross_command("yarn"),
        PackageManager::YarnBerry => cross_command("yarn"),
        PackageManager::Bun => cross_command("bun"),
      };

      cmd.arg("add").arg(&npm_spec);

      log::info!("Installing NPM dependency {npm_spec}...");
      let status = cmd
        .status()
        .with_context(|| format!("failed to run {manager}"))?;
      if !status.success() {
        anyhow::bail!("Failed to install NPM dependency");
      }
    }
  }

  let rust_code = if metadata.builder {
    if metadata.desktop_only {
      format!(
        r#"tauri::Builder::default()
    .setup(|app| {{
        #[cfg(desktop)]
        app.handle().plugin(tauri_plugin_{plugin_snake_case}::Builder::new().build());
        Ok(())
    }})
    "#,
      )
    } else {
      format!(
        r#"tauri::Builder::default()
    .setup(|app| {{
        app.handle().plugin(tauri_plugin_{plugin_snake_case}::Builder::new().build());
        Ok(())
    }})
    "#,
      )
    }
  } else if metadata.desktop_only {
    format!(
      r#"tauri::Builder::default()
    .setup(|app| {{
        #[cfg(desktop)]
        app.handle().plugin(tauri_plugin_{plugin_snake_case}::init());
        Ok(())
    }})
    "#,
    )
  } else {
    format!(
      r#"tauri::Builder::default().plugin(tauri_plugin_{plugin_snake_case}::init())
    "#,
    )
  };

  println!("You must enable the plugin in your Rust code:\n\n{rust_code}");

  Ok(())
}

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
