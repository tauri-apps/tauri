// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    framework::{infer_from_package_json as infer_framework, Framework},
    resolve_tauri_path, template, Logger,
  },
  VersionMetadata,
};
use std::{
  collections::BTreeMap,
  env::current_dir,
  fmt::Display,
  fs::{read_to_string, remove_dir_all},
  path::PathBuf,
  str::FromStr,
};

use crate::Result;
use anyhow::Context;
use clap::Parser;
use dialoguer::Input;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};
use serde::Deserialize;

const TEMPLATE_DIR: Dir<'_> = include_dir!("templates/app");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri project")]
pub struct Options {
  /// Skip prompting for values
  #[clap(long)]
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
  dist_dir: Option<String>,
  /// Url of your dev server
  #[clap(short = 'P', long)]
  dev_path: Option<String>,
}

#[derive(Deserialize)]
struct PackageJson {
  name: Option<String>,
  product_name: Option<String>,
}

#[derive(Default)]
struct InitDefaults {
  app_name: Option<String>,
  framework: Option<Framework>,
}

impl Options {
  fn load(mut self) -> Result<Self> {
    self.ci = self.ci || std::env::var("CI").is_ok();
    let package_json_path = PathBuf::from(&self.directory).join("package.json");

    let init_defaults = if package_json_path.exists() {
      let package_json_text = read_to_string(package_json_path)?;
      let package_json: PackageJson = serde_json::from_str(&package_json_text)?;
      let (framework, _) = infer_framework(&package_json_text);
      InitDefaults {
        app_name: package_json.product_name.or(package_json.name),
        framework,
      }
    } else {
      Default::default()
    };

    self.app_name = self.app_name.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      request_input(
        "What is your app name?",
        init_defaults.app_name.clone(),
        self.ci,
      )
    })?;

    self.window_title = self.window_title.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      request_input(
        "What should the window title be?",
        init_defaults.app_name.clone(),
        self.ci,
      )
    })?;

    self.dist_dir = self.dist_dir.map(|s| Ok(Some(s))).unwrap_or_else(|| request_input(
      r#"Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri/tauri.conf.json" file that will be created?"#,
      init_defaults.framework.as_ref().map(|f| f.dist_dir()),
      self.ci
    ))?;

    self.dev_path = self.dev_path.map(|s| Ok(Some(s))).unwrap_or_else(|| {
      request_input(
        "What is the url of your dev server?",
        init_defaults.framework.map(|f| f.dev_path()),
        self.ci,
      )
    })?;

    Ok(self)
  }
}

pub fn command(mut options: Options) -> Result<()> {
  options = options.load()?;
  let logger = Logger::new("tauri:init");

  let template_target_path = PathBuf::from(&options.directory).join("src-tauri");
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json"))?;

  if template_target_path.exists() && !options.force {
    logger.warn(format!(
      "Tauri dir ({:?}) not empty. Run `init --force` to overwrite.",
      template_target_path
    ));
  } else {
    let (tauri_dep, tauri_build_dep) = if let Some(tauri_path) = options.tauri_path {
      (
        format!(
          r#"{{  path = {:?}, features = [ "api-all" ] }}"#,
          resolve_tauri_path(&tauri_path, "core/tauri")
        ),
        format!(
          "{{  path = {:?} }}",
          resolve_tauri_path(&tauri_path, "core/tauri-build")
        ),
      )
    } else {
      (
        format!(
          r#"{{ version = "{}", features = [ "api-all" ] }}"#,
          metadata.tauri
        ),
        format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
      )
    };

    let _ = remove_dir_all(&template_target_path);
    let handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    data.insert("tauri_dep", to_json(tauri_dep));
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert(
      "dist_dir",
      to_json(options.dist_dir.unwrap_or_else(|| "../dist".to_string())),
    );
    data.insert(
      "dev_path",
      to_json(
        options
          .dev_path
          .unwrap_or_else(|| "http://localhost:4000".to_string()),
      ),
    );
    data.insert(
      "app_name",
      to_json(options.app_name.unwrap_or_else(|| "Tauri App".to_string())),
    );
    data.insert(
      "window_title",
      to_json(options.window_title.unwrap_or_else(|| "Tauri".to_string())),
    );

    template::render(&handlebars, &data, &TEMPLATE_DIR, &options.directory)
      .with_context(|| "failed to render Tauri template")?;
  }

  Ok(())
}

fn request_input<T>(prompt: &str, default: Option<T>, skip: bool) -> Result<Option<T>>
where
  T: Clone + FromStr + Display + ToString,
  T::Err: Display + std::fmt::Debug,
{
  if skip {
    Ok(default)
  } else {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut builder = Input::with_theme(&theme);
    builder.with_prompt(prompt);

    if let Some(v) = default {
      builder.default(v.clone());
      builder.with_initial_text(v.to_string());
    }

    builder.interact_text().map(Some).map_err(Into::into)
  }
}
