// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::PluginIosFramework;
use crate::{helpers::template, Result};
use clap::{Parser, Subcommand};
use handlebars::Handlebars;

use std::{
  collections::BTreeMap,
  env::current_dir,
  ffi::{OsStr, OsString},
  fs::{create_dir_all, File},
  path::{Component, PathBuf},
};

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Manage the iOS project for a Tauri plugin",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
}

#[derive(Debug, Parser)]
#[clap(about = "Initializes the iOS project for an existing Tauri plugin")]
pub struct InitOptions {
  /// Name of your Tauri plugin. Must match the current plugin's name.
  /// If not specified, it will be inferred from the current directory.
  plugin_name: Option<String>,
  /// The output directory.
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").to_string_lossy().into_owned())]
  out_dir: String,
  /// Type of framework to use for the iOS project.
  #[clap(long)]
  #[clap(default_value_t = PluginIosFramework::default())]
  ios_framework: PluginIosFramework,
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => {
      let plugin_name = match options.plugin_name {
        None => super::infer_plugin_name(std::env::current_dir()?)?,
        Some(name) => name,
      };

      let out_dir = PathBuf::from(options.out_dir);
      if out_dir.join("ios").exists() {
        return Err(anyhow::anyhow!("ios folder already exists"));
      }

      let handlebars = Handlebars::new();

      let mut data = BTreeMap::new();
      super::init::plugin_name_data(&mut data, &plugin_name);

      let ios_folder_name = match options.ios_framework {
        PluginIosFramework::Spm => OsStr::new("ios-spm"),
        PluginIosFramework::Xcode => OsStr::new("ios-xcode"),
      };

      let mut created_dirs = Vec::new();
      template::render_with_generator(
        &handlebars,
        &data,
        &super::init::TEMPLATE_DIR,
        &out_dir,
        &mut |path| {
          let mut components = path.components();
          let root = components.next().unwrap();
          if let Component::Normal(component) = root {
            if component == ios_folder_name {
              let folder_name = components.next().unwrap().as_os_str().to_string_lossy();
              let new_folder_name = folder_name.replace("{{ plugin_name }}", &plugin_name);
              let new_folder_name = OsString::from(&new_folder_name);

              let path = [
                Component::Normal(OsStr::new("ios")),
                Component::Normal(&new_folder_name),
              ]
              .into_iter()
              .chain(components)
              .collect::<PathBuf>();

              let path = out_dir.join(path);
              let parent = path.parent().unwrap().to_path_buf();
              if !created_dirs.contains(&parent) {
                create_dir_all(&parent)?;
                created_dirs.push(parent);
              }
              return File::create(path).map(Some);
            }
          }

          Ok(None)
        },
      )?;

      let metadata = super::init::crates_metadata()?;

      let cargo_toml_addition = format!(
        r#"
[build-dependencies]
tauri-build = "{}"
"#,
        metadata.tauri_build
      );
      let build_file = super::init::TEMPLATE_DIR
        .get_file("build.rs")
        .unwrap()
        .contents_utf8()
        .unwrap();
      let init_fn = format!(
        r#"
#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_{name});

pub fn init<R: Runtime>() -> TauriPlugin<R> {{
  Builder::new("{name}")
    .setup(|app| {{
      #[cfg(target_os = "ios")]
      app.register_ios_plugin(init_plugin_{name})?;
      Ok(())
    }})
    .build()
}}
"#,
        name = plugin_name,
      );

      log::info!("iOS project added");
      println!("You must add the following to the Cargo.toml file:\n{cargo_toml_addition}",);
      println!("You must add the following code to the build.rs file:\n\n{build_file}",);
      println!(
        "Your plugin's init function under src/lib.rs must initialize the iOS plugin:\n{init_fn}"
      );
    }
  }

  Ok(())
}
