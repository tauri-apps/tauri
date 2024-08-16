// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;
use colored::Colorize;
use regex::Regex;

use crate::{
  acl,
  helpers::{
    app_paths::{resolve_app_dir, tauri_dir},
    cargo,
    npm::PackageManager,
  },
  Result,
};

use std::process::Command;

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
  /// Don't format code with rustfmt
  #[clap(long)]
  pub no_fmt: bool,
}

pub fn command(options: Options) -> Result<()> {
  crate::helpers::app_paths::resolve();
  run(options)
}

pub fn run(options: Options) -> Result<()> {
  let (plugin, version) = options
    .plugin
    .split_once('@')
    .map(|(p, v)| (p, Some(v)))
    .unwrap_or((&options.plugin, None));

  let plugin_snake_case = plugin.replace('-', "_");
  let crate_name = format!("tauri-plugin-{plugin}");
  let npm_name = format!("@tauri-apps/plugin-{plugin}");

  let mut plugins = crate::helpers::plugins::known_plugins();
  let metadata = plugins.remove(plugin).unwrap_or_default();

  let app_dir = resolve_app_dir();
  let tauri_dir = tauri_dir();

  let target_str = metadata
    .desktop_only
    .then_some(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#)
    .or_else(|| {
      metadata
        .mobile_only
        .then_some(r#"cfg(any(target_os = "android", target_os = "ios"))"#)
    });

  cargo::install_one(cargo::CargoInstallOptions {
    name: &crate_name,
    version,
    branch: options.branch.as_deref(),
    rev: options.rev.as_deref(),
    tag: options.tag.as_deref(),
    cwd: Some(tauri_dir),
    target: target_str,
  })?;

  if !metadata.rust_only {
    if let Some(manager) = app_dir
      .map(PackageManager::from_project)
      .and_then(|managers| managers.into_iter().next())
    {
      let npm_spec = match (version, options.tag, options.rev, options.branch) {
        (Some(version), _, _, _) => {
          format!("{npm_name}@{version}")
        }
        (None, Some(tag), None, None) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{tag}")
        }
        (None, None, Some(rev), None) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{rev}")
        }
        (None, None, None, Some(branch)) => {
          format!("tauri-apps/tauri-plugin-{plugin}#{branch}")
        }
        (None, None, None, None) => npm_name,
        _ => anyhow::bail!("Only one of --tag, --rev and --branch can be specified"),
      };
      manager.install(&[npm_spec], tauri_dir)?;
    }

    let _ = acl::permission::add::command(acl::permission::add::Options {
      identifier: format!("{plugin}:default"),
      capability: None,
    });
  }

  // add plugin init code to main.rs or lib.rs
  let plugin_init_fn = if plugin == "stronghold" {
    "Builder::new(|pass| todo!()).build()"
  } else if plugin == "localhost" {
    "Builder::new(todo!()).build()"
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

      if !options.no_fmt {
        // reformat code with rustfmt
        log::info!("Running `cargo fmt`...");
        let _ = Command::new("cargo")
          .arg("fmt")
          .current_dir(tauri_dir)
          .status();
      }

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
