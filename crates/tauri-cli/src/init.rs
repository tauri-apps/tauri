// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  VersionMetadata,
  helpers::{
    framework::{Framework, infer_from_package_json as infer_framework},
    npm::PackageManager,
    prompts, resolve_tauri_path, template,
  },
};
use std::{
  collections::BTreeMap,
  env::current_dir,
  fs::{read_to_string, remove_dir_all},
  path::PathBuf,
};

use crate::{
  Result,
  error::{Context, ErrorExt},
};
use clap::Parser;
use handlebars::{Handlebars, to_json};
use include_dir::{Dir, include_dir};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/app");
const NODE_TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/bindings/node");
const DENO_TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/bindings/deno");
const PYTHON_TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/bindings/python");
const C_TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/bindings/c");
const TAURI_CONF_TEMPLATE: &str = include_str!("../templates/tauri.conf.json");
const BINDINGS_TAURI_CONF_TEMPLATE: &str = include_str!("../templates/bindings.tauri.conf.json");
const C_TAURI_CONF_TEMPLATE: &str = include_str!("../templates/c.tauri.conf.json");

/// The language a scaffolded Tauri backend is written in. `Rust` is the classic
/// project (a cargo crate); the others are `tauri-ffi` bindings projects driven
/// by a `build > runner` command (Node.js, Deno, Python) or linked directly (C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Language {
  #[default]
  Rust,
  #[value(name = "node", alias = "nodejs", alias = "js")]
  Node,
  Deno,
  Python,
  C,
}

impl Language {
  const ALL: [Language; 5] = [
    Language::Rust,
    Language::Node,
    Language::Deno,
    Language::Python,
    Language::C,
  ];

  fn label(self) -> &'static str {
    match self {
      Language::Rust => "Rust",
      Language::Node => "Node.js",
      Language::Deno => "Deno",
      Language::Python => "Python",
      Language::C => "C",
    }
  }

  /// The `build > runner` command scaffolded into the config, or `None` for
  /// languages compiled directly (Rust, C).
  fn runner(self) -> Option<&'static str> {
    match self {
      Language::Node => Some(r#"{ "cmd": "node", "args": ["main.js"] }"#),
      Language::Deno => Some(r#"{ "cmd": "deno", "args": ["run", "-A", "main.ts"] }"#),
      Language::Python => Some(r#"{ "cmd": "python3", "args": ["main.py"] }"#),
      Language::Rust | Language::C => None,
    }
  }
}

impl std::fmt::Display for Language {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.label())
  }
}

#[derive(Debug, Parser)]
#[clap(about = "Initialize a Tauri project in an existing directory")]
pub struct Options {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  /// Force init to overwrite the src-tauri folder
  #[clap(short, long)]
  force: bool,
  /// Language the Tauri backend is written in
  #[clap(long, value_enum)]
  language: Option<Language>,
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
    if self.language.is_none() {
      let selected = prompts::select(
        "What language will your Tauri backend use?",
        &Language::ALL,
        0,
        self.ci,
      )?;
      self.language = Some(Language::ALL[selected]);
    }

    let package_json_path = PathBuf::from(&self.directory).join("package.json");

    let init_defaults = if package_json_path.exists() {
      let package_json_text =
        read_to_string(&package_json_path).fs_context("failed to read", &package_json_path)?;
      let package_json: crate::PackageJson =
        serde_json::from_str(&package_json_text).context("failed to parse JSON")?;
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
  let language = options.language.unwrap_or_default();

  let template_target_path = PathBuf::from(&options.directory).join("src-tauri");
  if template_target_path.exists() && !options.force {
    log::warn!(
      "Tauri dir ({:?}) not empty. Run `init --force` to overwrite.",
      template_target_path
    );
    return Ok(());
  }

  let _ = remove_dir_all(&template_target_path);
  let mut handlebars = Handlebars::new();
  handlebars.register_escape_fn(handlebars::no_escape);

  let mut data = BTreeMap::new();
  data.insert(
    "frontend_dist",
    to_json(options.frontend_dist.as_deref().unwrap_or("../dist")),
  );
  data.insert("dev_url", to_json(options.dev_url.clone()));
  data.insert(
    "app_name",
    to_json(options.app_name.as_deref().unwrap_or("Tauri App")),
  );
  data.insert(
    "window_title",
    to_json(options.window_title.as_deref().unwrap_or("Tauri")),
  );
  data.insert(
    "before_dev_command",
    to_json(options.before_dev_command.clone()),
  );
  data.insert(
    "before_build_command",
    to_json(options.before_build_command.clone()),
  );

  // Fill in the language-specific handlebars data.
  match language {
    Language::Rust => insert_rust_deps(&mut data, &options)?,
    Language::Node | Language::Deno | Language::Python => {
      data.insert("runner", to_json(language.runner().expect("has a runner")));
    }
    Language::C => {}
  }

  let config = render_config(&handlebars, config_template(language), &data)?;
  data.insert("tauri_config", to_json(config));

  template::render(&handlebars, &data, template_dir(language), &options.directory)
    .with_context(|| "failed to render Tauri template")?;

  Ok(())
}

/// The `tauri.conf.json` template for a language.
fn config_template(language: Language) -> &'static str {
  match language {
    Language::Rust => TAURI_CONF_TEMPLATE,
    Language::C => C_TAURI_CONF_TEMPLATE,
    Language::Node | Language::Deno | Language::Python => BINDINGS_TAURI_CONF_TEMPLATE,
  }
}

/// The directory of files scaffolded into `<directory>/src-tauri` for a language.
fn template_dir(language: Language) -> &'static Dir<'static> {
  match language {
    Language::Rust => &TEMPLATE_DIR,
    Language::Node => &NODE_TEMPLATE_DIR,
    Language::Deno => &DENO_TEMPLATE_DIR,
    Language::Python => &PYTHON_TEMPLATE_DIR,
    Language::C => &C_TEMPLATE_DIR,
  }
}

/// Inserts the `tauri`/`tauri-build`/… dependency specs a Rust project's
/// `Cargo.toml` template expects, resolving to local paths when `--tauri-path`
/// is given.
fn insert_rust_deps(
  data: &mut BTreeMap<&str, serde_json::Value>,
  options: &Options,
) -> Result<()> {
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata-v2.json"))
    .context("failed to parse version metadata")?;

  let (tauri_dep, tauri_build_dep, tauri_utils_dep, tauri_plugin_dep) =
    if let Some(tauri_path) = &options.tauri_path {
      (
        format!(
          r#"{{  path = {:?} }}"#,
          resolve_tauri_path(tauri_path, "crates/tauri")
        ),
        format!(
          "{{  path = {:?} }}",
          resolve_tauri_path(tauri_path, "crates/tauri-build")
        ),
        format!(
          "{{  path = {:?} }}",
          resolve_tauri_path(tauri_path, "crates/tauri-utils")
        ),
        format!(
          "{{  path = {:?} }}",
          resolve_tauri_path(tauri_path, "crates/tauri-plugin")
        ),
      )
    } else {
      (
        format!(r#"{{ version = "{}" }}"#, metadata.tauri),
        format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
        r#"{{ version = "2" }}"#.to_string(),
        r#"{{ version = "2" }}"#.to_string(),
      )
    };

  data.insert("tauri_dep", to_json(tauri_dep));
  if options.tauri_path.is_some() {
    data.insert("patch_tauri_dep", to_json(true));
  }
  data.insert("tauri_build_dep", to_json(tauri_build_dep));
  data.insert("tauri_utils_dep", to_json(tauri_utils_dep));
  data.insert("tauri_plugin_dep", to_json(tauri_plugin_dep));
  Ok(())
}

/// Renders a `tauri.conf.json` template, then (when the CLI is the Node.js
/// distribution) rewrites `$schema` to the local `config.schema.json` so
/// editors resolve it offline. Returns the pretty-printed config.
fn render_config(
  handlebars: &Handlebars<'_>,
  template: &str,
  data: &BTreeMap<&str, serde_json::Value>,
) -> Result<String> {
  let mut config: serde_json::Value = serde_json::from_str(
    &handlebars
      .render_template(template, data)
      .expect("Failed to render tauri.conf.json template"),
  )
  .expect("rendered tauri.conf.json is not valid JSON");

  if option_env!("TARGET") == Some("node") {
    if let Some(schema) = node_config_schema_path() {
      let mut map = serde_json::Map::default();
      map.insert("$schema".into(), serde_json::Value::String(schema));
      json_patch::merge(&mut config, &serde_json::Value::Object(map));
    }
  }

  Ok(serde_json::to_string_pretty(&config).expect("failed to serialize tauri.conf.json"))
}

/// The relative path from the scaffolded `src-tauri` to the Node.js CLI's
/// bundled `config.schema.json`, if the CLI is installed in a nearby
/// `node_modules` (searching up to three parent directories).
fn node_config_schema_path() -> Option<String> {
  let mut dir = current_dir().expect("failed to read cwd");
  let cli_path = "node_modules/@tauri-apps/cli";

  // only go up three folders max
  for count in 0..=2 {
    if dir.join(cli_path).exists() {
      let mut path = PathBuf::from("..");
      for _ in 0..count {
        path.push("..");
      }
      path.push(cli_path);
      path.push("config.schema.json");
      return Some(path.display().to_string().replace('\\', "/"));
    }
    match dir.parent() {
      Some(parent) => dir = parent.to_path_buf(),
      None => break,
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::Path;

  /// Every language's scaffolded `tauri.conf.json` must parse against the real
  /// `Config` deserializer (which rejects unknown fields), and bindings
  /// languages must carry a runnable `build > runner`.
  #[test]
  fn scaffolded_configs_parse() {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    for language in Language::ALL {
      let mut data: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
      data.insert("frontend_dist", to_json("../dist"));
      data.insert("dev_url", to_json("http://localhost:3000"));
      data.insert("app_name", to_json("Test App"));
      data.insert("window_title", to_json("Test"));
      data.insert("before_dev_command", to_json("npm run dev"));
      data.insert("before_build_command", to_json("npm run build"));
      if let Some(runner) = language.runner() {
        data.insert("runner", to_json(runner));
      }

      let rendered = render_config(&handlebars, config_template(language), &data)
        .unwrap_or_else(|e| panic!("{language} config failed to render: {e}"));
      let config =
        tauri_utils::config::parse::parse_json(&rendered, Path::new("tauri.conf.json"))
          .unwrap_or_else(|e| panic!("{language} config failed to parse: {e}\n{rendered}"));

      // Node/Deno/Python are driven by `build > runner`; Rust and C are not.
      assert_eq!(
        config.build.runner.is_some(),
        language.runner().is_some(),
        "{language} runner presence mismatch"
      );
    }
  }
}
