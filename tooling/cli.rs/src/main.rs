// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

pub use anyhow::Result;
use clap::{crate_version, load_yaml, App, AppSettings, ArgMatches};
use dialoguer::Input;
use serde::Deserialize;

mod build;
mod dev;
mod helpers;
mod info;
mod init;
mod interface;
mod sign;

// temporary fork from https://github.com/mitsuhiko/console until 0.14.1+ release
#[allow(dead_code)]
mod console;
// temporary fork from https://github.com/mitsuhiko/dialoguer until 0.8.0+ release
#[allow(dead_code)]
mod dialoguer;

use helpers::framework::{infer_from_package_json as infer_framework, Framework};

use std::{env::current_dir, fs::read_to_string, path::PathBuf};

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

macro_rules! value_or_prompt {
  ($init_runner: ident, $setter_fn: ident, $value: ident, $ci: ident, $prompt_message: expr, $prompt_default: expr) => {{
    let mut init_runner = $init_runner;
    if let Some(value) = $value {
      init_runner = init_runner.$setter_fn(value);
    } else if !$ci {
      let mut builder = Input::<String>::new();
      builder.with_prompt($prompt_message);
      if let Some(default) = $prompt_default {
        builder.default(default);
      }
      let input = builder.interact_text()?;
      init_runner = init_runner.$setter_fn(input);
    }
    init_runner
  }};
}

fn init_command(matches: &ArgMatches) -> Result<()> {
  let force = matches.is_present("force");
  let directory = matches.value_of("directory");
  let tauri_path = matches.value_of("tauri-path");
  let app_name = matches.value_of("app-name");
  let window_title = matches.value_of("window-title");
  let dist_dir = matches.value_of("dist-dir");
  let dev_path = matches.value_of("dev-path");
  let ci = matches.is_present("ci") || std::env::var("CI").is_ok();

  let mut init_runner = init::Init::new();
  if force {
    init_runner = init_runner.force();
  }
  let base_directory = if let Some(directory) = directory {
    init_runner = init_runner.directory(directory);
    PathBuf::from(directory)
  } else {
    current_dir().expect("failed to read cwd")
  };
  if let Some(tauri_path) = tauri_path {
    init_runner = init_runner.tauri_path(tauri_path);
  }

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

  init_runner = value_or_prompt!(
    init_runner,
    app_name,
    app_name,
    ci,
    "What is your app name?",
    init_defaults.app_name.clone()
  );
  init_runner = value_or_prompt!(
    init_runner,
    window_title,
    window_title,
    ci,
    "What should the window title be?",
    init_defaults.app_name.clone()
  );
  init_runner = value_or_prompt!(
    init_runner,
    dist_dir,
    dist_dir,
    ci,
    r#"Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri" folder that will be created?"#,
    init_defaults.framework.as_ref().map(|f| f.dist_dir())
  );
  init_runner = value_or_prompt!(
    init_runner,
    dev_path,
    dev_path,
    ci,
    "What is the url of your dev server?",
    init_defaults.framework.map(|f| f.dev_path())
  );

  init_runner.run()
}

fn dev_command(matches: &ArgMatches) -> Result<()> {
  let runner = matches.value_of("runner");
  let target = matches.value_of("target");
  let features: Vec<String> = matches
    .values_of("features")
    .map(|a| a.into_iter().map(|v| v.to_string()).collect())
    .unwrap_or_default();
  let exit_on_panic = matches.is_present("exit-on-panic");
  let config = matches.value_of("config");
  let args: Vec<String> = matches
    .values_of("args")
    .map(|a| a.into_iter().map(|v| v.to_string()).collect())
    .unwrap_or_default();
  let release_mode = matches.is_present("release");

  let mut dev_runner = dev::Dev::new()
    .exit_on_panic(exit_on_panic)
    .args(args)
    .features(features)
    .release_mode(release_mode);

  if let Some(runner) = runner {
    dev_runner = dev_runner.runner(runner.to_string());
  }
  if let Some(target) = target {
    dev_runner = dev_runner.target(target.to_string());
  }
  if let Some(config) = config {
    dev_runner = dev_runner.config(config.to_string());
  }

  dev_runner.run()
}

fn build_command(matches: &ArgMatches) -> Result<()> {
  let runner = matches.value_of("runner");
  let target = matches.value_of("target");
  let features: Vec<String> = matches
    .values_of("features")
    .map(|a| a.into_iter().map(|v| v.to_string()).collect())
    .unwrap_or_default();
  let debug = matches.is_present("debug");
  let verbose = matches.is_present("verbose");
  let bundles = matches.values_of_lossy("bundle");
  let config = matches.value_of("config");

  let mut build_runner = build::Build::new().features(features);
  if let Some(runner) = runner {
    build_runner = build_runner.runner(runner.to_string());
  }
  if let Some(target) = target {
    build_runner = build_runner.target(target.to_string());
  }
  if debug {
    build_runner = build_runner.debug();
  }
  if verbose {
    build_runner = build_runner.verbose();
  }
  if let Some(bundles) = bundles {
    build_runner = build_runner.bundles(bundles);
  }
  if let Some(config) = config {
    build_runner = build_runner.config(config.to_string());
  }

  build_runner.run()
}

fn info_command() -> Result<()> {
  info::Info::new().run()
}

fn sign_command(matches: &ArgMatches) -> Result<()> {
  let private_key = matches.value_of("private-key");
  let private_key_path = matches.value_of("private-key-path");
  let file = matches.value_of("sign-file");
  let password = matches.value_of("password");
  let no_password = matches.is_present("no-password");
  let write_keys = matches.value_of("write-keys");
  let force = matches.is_present("force");

  // generate keypair
  if matches.is_present("generate") {
    let mut keygen_runner = sign::KeyGenerator::new();

    if no_password {
      keygen_runner = keygen_runner.empty_password();
    }

    if force {
      keygen_runner = keygen_runner.force();
    }

    if let Some(write_keys) = write_keys {
      keygen_runner = keygen_runner.output_path(write_keys);
    }

    if let Some(password) = password {
      keygen_runner = keygen_runner.password(password);
    }

    return keygen_runner.generate_keys();
  }

  // sign our binary / archive
  let mut sign_runner = sign::Signer::new();
  if let Some(private_key) = private_key {
    sign_runner = sign_runner.private_key(private_key);
  }

  if let Some(private_key_path) = private_key_path {
    sign_runner = sign_runner.private_key_path(private_key_path);
  }

  if let Some(file) = file {
    sign_runner = sign_runner.file_to_sign(file);
  }

  if let Some(password) = password {
    sign_runner = sign_runner.password(password);
  }

  if no_password {
    sign_runner = sign_runner.empty_password();
  }

  sign_runner.run()
}

fn main() -> Result<()> {
  let yaml = load_yaml!("cli.yml");
  let app = App::from(yaml)
    .version(crate_version!())
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::SubcommandRequired);
  let app_matches = app.get_matches();
  let matches = app_matches.subcommand_matches("tauri").unwrap();

  if let Some(matches) = matches.subcommand_matches("init") {
    init_command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("dev") {
    dev_command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("build") {
    build_command(matches)?;
  } else if matches.subcommand_matches("info").is_some() {
    info_command()?;
  } else if let Some(matches) = matches.subcommand_matches("sign") {
    sign_command(matches)?;
  }

  Ok(())
}
