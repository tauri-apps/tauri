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
use clap::ArgMatches;
use dialoguer::Input;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};
use serde::Deserialize;

const TEMPLATE_DIR: Dir<'_> = include_dir!("templates/app");

pub struct InitOptions {
  force: bool,
  directory: PathBuf,
  tauri_path: Option<PathBuf>,
  app_name: Option<String>,
  window_title: Option<String>,
  dist_dir: Option<String>,
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

impl TryFrom<&ArgMatches> for InitOptions {
  type Error = anyhow::Error;
  fn try_from(matches: &ArgMatches) -> Result<Self> {
    let force = matches.is_present("force");
    let directory = matches.value_of("directory");
    let tauri_path = matches.value_of("tauri-path");
    let app_name = matches.value_of("app-name");
    let window_title = matches.value_of("window-title");
    let dist_dir = matches.value_of("dist-dir");
    let dev_path = matches.value_of("dev-path");
    let ci = matches.is_present("ci") || std::env::var("CI").is_ok();

    let base_directory = directory
      .map(PathBuf::from)
      .unwrap_or(current_dir().expect("failed to read cwd"));

    let package_json_path = base_directory.join("package.json");

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

    Ok(Self {
            force,
            directory: base_directory,
            tauri_path: tauri_path.map(PathBuf::from),
            app_name: app_name.map(ToString::to_string)
                .or(Some(request_input(
                    "What is your app name?",
                    init_defaults.app_name.clone(),
                    ci)?)
                ),
            window_title: window_title.map(ToString::to_string)
                .or(Some(request_input(
                    "What should the window title be?",
                    init_defaults.app_name.clone(),
                    ci)?)
                ),
            dist_dir: dist_dir.map(ToString::to_string)
                .or(Some(request_input(
                    r#"Whe re are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri/tauri.conf.json" file that will be created?"#,
                    init_defaults.framework.as_ref().map(|f| f.dist_dir()),
                    ci)?)
                ),
            dev_path: dev_path.map(ToString::to_string)
                .or(Some(request_input(
                    "What is the url of your dev server?",
                    init_defaults.framework.map(|f| f.dev_path()),
                    ci)?)
                ),
        })
  }
}

pub fn command(matches: &ArgMatches) -> Result<()> {
  let logger = Logger::new("tauri:init");
  let options = InitOptions::try_from(matches)?;

  let template_target_path = options.directory.join("src-tauri");
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

fn request_input<T>(prompt: &str, default: Option<T>, skip: bool) -> Result<T>
where
  T: Clone + FromStr + Display,
  T::Err: Display + std::fmt::Debug,
{
  if skip {
    default.ok_or(anyhow::Error::msg("missing input"))
  } else {
    let mut builder = Input::new();
    builder.with_prompt(prompt);

    if default.is_some() {
      builder.default(default.unwrap());
    }

    builder.interact_text().map_err(Into::into)
  }
}
