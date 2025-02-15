// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    framework::{infer_from_package_json as infer_framework, Framework},
    npm::PackageManager,
    prompts, resolve_tauri_path, template,
  },
  VersionMetadata,
};
use std::{
  collections::BTreeMap,
  env::current_dir,
  fs::{read_to_string, remove_dir_all},
  path::PathBuf,
};

use crate::Result;
use anyhow::Context;
use clap::Parser;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/app");
const TAURI_CONF_TEMPLATE: &str = include_str!("../templates/tauri.conf.json");

#[derive(Debug, Parser)]
#[clap(about = "Initialize a Tauri project in an existing directory")]
pub struct Options {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  /// Force init to overwrite the src-tauri folder
  #[clap(short, long)]
  force: bool,
  /// Enables logging
  #[clap(short, long)]
  log: bool,
  /// Set target directory for init
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
  directory: String,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  tauri_path: Option<PathBuf>,
  /// Name of your Tauri application
  #[clap(short = 'A', long)]
  app_name: Option<String>,
  /// Window title of your Tauri application
  #[clap(short = 'W', long)]
  window_title: Option<String>,
  /// Web assets location, relative to <project-dir>/src-tauri
  #[clap(short = 'D', long)]
  frontend_dist: Option<String>,
  /// Url of your dev server
  #[clap(short = 'P', long)]
  dev_url: Option<String>,
  /// A shell command to run before `tauri dev` kicks in.
  #[clap(long)]
  before_dev_command: Option<String>,
  /// A shell command to run before `tauri build` kicks in.
  #[clap(long)]
  before_build_command: Option<String>,
}

#[derive(Default)]
struct InitDefaults {
  app_name: Option<String>,
  framework: Option<Framework>,
}

impl Options {
  fn load(mut self) -> Result<Self> {
    let package_json_path = PathBuf::from(&self.directory).join("package.json");

    let init_defaults = if package_json_path.exists() {
      let package_json_text = read_to_string(package_json_path)?;
      let package_json: crate::PackageJson = serde_json::from_str(&package_json_text)?;
      let (framework, _) = infer_framework(&package_json_text);
      InitDefaults {
        app_name: package_json.product_name.or(package_json.name),
        framework,
      }
    } else {
      Default::default()
    };

    self.app_name = self.app_name.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      prompts::input(
        "What is your app name?",
        Some(
          init_defaults
            .app_name
            .clone()
            .unwrap_or_else(|| "Tauri App".to_string()),
        ),
        self.ci,
        true,
      )
    })?;

    self.window_title = self.window_title.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      prompts::input(
        "What should the window title be?",
        Some(
          init_defaults
            .app_name
            .clone()
            .unwrap_or_else(|| "Tauri".to_string()),
        ),
        self.ci,
        true,
      )
    })?;

    self.frontend_dist = self.frontend_dist.map(|s| Ok(Some(s))).unwrap_or_else(|| prompts::input(
      r#"Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri/tauri.conf.json" file that will be created?"#,
      init_defaults.framework.as_ref().map(|f| f.frontend_dist()),
      self.ci,
      false,
    ))?;

    self.dev_url = self.dev_url.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      prompts::input(
        "What is the url of your dev server?",
        init_defaults.framework.map(|f| f.dev_url()),
        self.ci,
        true,
      )
    })?;

    let detected_package_manager = PackageManager::from_project(&self.directory);

    self.before_dev_command = self
      .before_dev_command
      .map(|s| Ok(Some(s)))
      .unwrap_or_else(|| {
        prompts::input(
          "What is your frontend dev command?",
          Some(default_dev_command(detected_package_manager).into()),
          self.ci,
          true,
        )
      })?;

    self.before_build_command = self
      .before_build_command
      .map(|s| Ok(Some(s)))
      .unwrap_or_else(|| {
        prompts::input(
          "What is your frontend build command?",
          Some(default_build_command(detected_package_manager).into()),
          self.ci,
          true,
        )
      })?;

    Ok(self)
  }
}

fn default_dev_command(pm: PackageManager) -> &'static str {
  match pm {
    PackageManager::Yarn => "yarn dev",
    PackageManager::YarnBerry => "yarn dev",
    PackageManager::Npm => "npm run dev",
    PackageManager::Pnpm => "pnpm dev",
    PackageManager::Bun => "bun dev",
    PackageManager::Deno => "deno task dev",
  }
}

fn default_build_command(pm: PackageManager) -> &'static str {
  match pm {
    PackageManager::Yarn => "yarn build",
    PackageManager::YarnBerry => "yarn build",
    PackageManager::Npm => "npm run build",
    PackageManager::Pnpm => "pnpm build",
    PackageManager::Bun => "bun build",
    PackageManager::Deno => "deno task build",
  }
}

pub fn command(mut options: Options) -> Result<()> {
  options = options.load()?;

  let template_target_path = PathBuf::from(&options.directory).join("src-tauri");
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata-v2.json"))?;

  if template_target_path.exists() && !options.force {
    log::warn!(
      "Tauri dir ({:?}) not empty. Run `init --force` to overwrite.",
      template_target_path
    );
  } else {
    let (tauri_dep, tauri_build_dep) = if let Some(tauri_path) = &options.tauri_path {
      (
        format!(
          r#"{{  path = {:?} }}"#,
          resolve_tauri_path(tauri_path, "crates/tauri")
        ),
        format!(
          "{{  path = {:?} }}",
          resolve_tauri_path(tauri_path, "crates/tauri-build")
        ),
      )
    } else {
      (
        format!(r#"{{ version = "{}" }}"#, metadata.tauri),
        format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
      )
    };

    let _ = remove_dir_all(&template_target_path);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let mut data = BTreeMap::new();
    data.insert("tauri_dep", to_json(tauri_dep));
    if options.tauri_path.is_some() {
      data.insert("patch_tauri_dep", to_json(true));
    }
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert(
      "frontend_dist",
      to_json(options.frontend_dist.as_deref().unwrap_or("../dist")),
    );
    data.insert("dev_url", to_json(options.dev_url));
    data.insert(
      "app_name",
      to_json(options.app_name.as_deref().unwrap_or("Tauri App")),
    );
    data.insert(
      "window_title",
      to_json(options.window_title.as_deref().unwrap_or("Tauri")),
    );
    data.insert("before_dev_command", to_json(options.before_dev_command));
    data.insert(
      "before_build_command",
      to_json(options.before_build_command),
    );

    let mut config = serde_json::from_str(
      &handlebars
        .render_template(TAURI_CONF_TEMPLATE, &data)
        .expect("Failed to render tauri.conf.json template"),
    )
    .unwrap();
    if option_env!("TARGET") == Some("node") {
      let mut dir = current_dir().expect("failed to read cwd");
      let mut count = 0;
      let mut cli_node_module_path = None;
      let cli_path = "node_modules/@tauri-apps/cli";

      // only go up three folders max
      while count <= 2 {
        let test_path = dir.join(cli_path);
        if test_path.exists() {
          let mut node_module_path = PathBuf::from("..");
          for _ in 0..count {
            node_module_path.push("..");
          }
          node_module_path.push(cli_path);
          node_module_path.push("config.schema.json");
          cli_node_module_path.replace(node_module_path);
          break;
        }
        count += 1;
        match dir.parent() {
          Some(parent) => {
            dir = parent.to_path_buf();
          }
          None => break,
        }
      }

      if let Some(cli_node_module_path) = cli_node_module_path {
        let mut map = serde_json::Map::default();
        map.insert(
          "$schema".into(),
          serde_json::Value::String(
            cli_node_module_path
              .display()
              .to_string()
              .replace('\\', "/"),
          ),
        );
        let merge_config = serde_json::Value::Object(map);
        json_patch::merge(&mut config, &merge_config);
      }
    }

    data.insert(
      "tauri_config",
      to_json(serde_json::to_string_pretty(&config).unwrap()),
    );

    template::render(&handlebars, &data, &TEMPLATE_DIR, &options.directory)
      .with_context(|| "failed to render Tauri template")?;
  }

  Ok(())
}
