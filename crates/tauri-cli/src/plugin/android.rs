// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Result, error::Context, helpers::template};
use clap::{Parser, Subcommand};
use handlebars::Handlebars;
use heck::ToKebabCase;

use std::{
  collections::BTreeMap,
  env::current_dir,
  ffi::OsStr,
  path::{Component, PathBuf},
};

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Manage the Android project for a Tauri plugin",
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
#[clap(about = "Initializes the Android project for an existing Tauri plugin")]
pub struct InitOptions {
  /// Name of your Tauri plugin. Must match the current plugin's name.
  /// If not specified, it will be inferred from the current directory.
  plugin_name: Option<String>,
  /// The output directory.
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").to_string_lossy().into_owned())]
  out_dir: String,
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => {
      let plugin_name = match options.plugin_name {
        None => super::infer_plugin_name(
          std::env::current_dir().context("failed to get current directory")?,
        )?,
        Some(name) => name,
      };

      let out_dir = PathBuf::from(options.out_dir);
      if out_dir.join("android").exists() {
        crate::error::bail!("Android folder already exists");
      }

      let plugin_id = super::init::prompt_android_package_id(&plugin_name)?;

      let handlebars = Handlebars::new();

      let mut data = BTreeMap::new();
      super::init::plugin_name_data(&mut data, &plugin_name);
      data.insert("android_package_id", handlebars::to_json(&plugin_id));

      let mut created_dirs = Vec::new();
      template::render_with_generator(
        &handlebars,
        &data,
        &super::init::TEMPLATE_DIR,
        &out_dir,
        &mut |path| {
          let mut components = path.components();
          let root = components.next().unwrap();
          if let Component::Normal(component) = root
            && component == OsStr::new("android")
          {
            return super::init::generate_android_out_file(
              &path,
              &out_dir,
              &plugin_id.replace('.', "/"),
              &mut created_dirs,
            );
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
      let builder_name = plugin_name.to_kebab_case();
      let init_fn = format!(
        r#"
pub fn init<R: Runtime>() -> TauriPlugin<R> {{
  Builder::new("{builder_name}")
    .setup(|app, api| {{
      #[cfg(target_os = "android")]
      let handle = api.register_android_plugin("{plugin_id}", "ExamplePlugin")?;
      Ok(())
    }})
    .build()
}}
"#
      );

      log::info!("Android project added");
      println!("You must add the following to the Cargo.toml file:\n{cargo_toml_addition}",);
      println!("You must add the following code to the build.rs file:\n\n{build_file}",);
      println!(
        "Your plugin's init function under src/lib.rs must initialize the Android plugin:\n{init_fn}"
      );
    }
  }

  Ok(())
}
