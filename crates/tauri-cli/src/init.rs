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
  io::IsTerminal,
  path::{Path, PathBuf},
};

use crate::{
  Result,
  error::{Context, ErrorExt},
};
use clap::Parser;
use handlebars::{Handlebars, to_json};
use include_dir::{Dir, include_dir};

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
    if !std::io::stdin().is_terminal() {
      self.ci = true;
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
          "What command should Tauri run before `tauri dev` to start your frontend? (leave empty if not needed)",
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
          "What command should Tauri run before `tauri build` to build your frontend? (leave empty if not needed)",
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
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata-v2.json"))
    .context("failed to parse version metadata")?;

  if template_target_path.exists() && !options.force {
    log::warn!(
      "Tauri dir ({:?}) not empty. Run `init --force` to overwrite.",
      template_target_path
    );
  } else {
    let _ = remove_dir_all(&template_target_path);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let mut data = tauri_dependencies_data(options.tauri_path.as_deref(), &metadata);
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

/// Builds the template variables for the Tauri crate dependencies of the generated `Cargo.toml`.
///
/// `tauri_dep` and `tauri_build_dep` are always set.
///
/// When `tauri_path` is provided the dependencies point to the crates inside that directory
/// and `patch_tauri_dep`, `tauri_utils_dep` and `tauri_plugin_dep` are also set so the template
/// renders a `[patch.crates-io]` section overriding the Tauri crates pulled in by other dependencies.
/// Without a path the template never renders that section, so those variables are not needed.
fn tauri_dependencies_data(
  tauri_path: Option<&Path>,
  metadata: &VersionMetadata,
) -> BTreeMap<&'static str, serde_json::Value> {
  let mut data = BTreeMap::new();
  if let Some(tauri_path) = tauri_path {
    let path_dep = |crate_dir: &str| {
      to_json(format!(
        "{{  path = {:?} }}",
        resolve_tauri_path(tauri_path, crate_dir)
      ))
    };
    data.insert("tauri_dep", path_dep("crates/tauri"));
    data.insert("tauri_build_dep", path_dep("crates/tauri-build"));
    data.insert("patch_tauri_dep", to_json(true));
    data.insert("tauri_utils_dep", path_dep("crates/tauri-utils"));
    data.insert("tauri_plugin_dep", path_dep("crates/tauri-plugin"));
  } else {
    data.insert(
      "tauri_dep",
      to_json(format!(r#"{{ version = "{}" }}"#, metadata.tauri)),
    );
    data.insert(
      "tauri_build_dep",
      to_json(format!(r#"{{ version = "{}" }}"#, metadata.tauri_build)),
    );
  }
  data
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The version metadata shipped with the CLI, the same source `command` uses.
  fn metadata() -> VersionMetadata {
    serde_json::from_str(include_str!("../metadata-v2.json"))
      .expect("failed to parse version metadata")
  }

  /// Renders the app `Cargo.toml` template with the same handlebars setup used by `command`.
  fn render_manifest(data: &BTreeMap<&'static str, serde_json::Value>) -> toml::Table {
    let template = TEMPLATE_DIR
      .get_file("src-tauri/Cargo.crate-manifest")
      .expect("app template is missing src-tauri/Cargo.crate-manifest")
      .contents_utf8()
      .expect("Cargo manifest template is not UTF-8");

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    let rendered = handlebars
      .render_template(template, data)
      .expect("failed to render Cargo manifest template");

    assert!(
      !rendered.contains("{{") && !rendered.contains("}}"),
      "rendered manifest contains handlebars braces:\n{rendered}"
    );

    toml::from_str(&rendered)
      .unwrap_or_else(|e| panic!("rendered manifest is not valid TOML: {e}\n{rendered}"))
  }

  fn dependency<'a>(manifest: &'a toml::Table, section: &str, name: &str) -> &'a toml::Table {
    manifest
      .get(section)
      .and_then(|s| s.get(name))
      .and_then(|d| d.as_table())
      .unwrap_or_else(|| panic!("[{section}] is missing the `{name}` table"))
  }

  fn assert_path_dependency(dep: &toml::Table, tauri_path: &Path, crate_dir: &str) {
    assert!(
      !dep.contains_key("version"),
      "path dependency must not carry a version: {dep:?}"
    );
    let path = dep
      .get("path")
      .and_then(|p| p.as_str())
      .unwrap_or_else(|| panic!("dependency is missing a string `path`: {dep:?}"));
    assert_eq!(Path::new(path), resolve_tauri_path(tauri_path, crate_dir));
  }

  fn assert_manifest_uses_path(manifest: &toml::Table, tauri_path: &Path) {
    assert_path_dependency(
      dependency(manifest, "dependencies", "tauri"),
      tauri_path,
      "crates/tauri",
    );
    assert_path_dependency(
      dependency(manifest, "build-dependencies", "tauri-build"),
      tauri_path,
      "crates/tauri-build",
    );

    let patch = manifest
      .get("patch")
      .and_then(|p| p.get("crates-io"))
      .and_then(|p| p.as_table())
      .expect("[patch.crates-io] must be rendered for path dependencies");
    assert_eq!(
      patch.len(),
      3,
      "unexpected [patch.crates-io] entries: {patch:?}"
    );
    for (name, crate_dir) in [
      ("tauri", "crates/tauri"),
      ("tauri-utils", "crates/tauri-utils"),
      ("tauri-plugin", "crates/tauri-plugin"),
    ] {
      let dep = patch
        .get(name)
        .and_then(|d| d.as_table())
        .unwrap_or_else(|| panic!("[patch.crates-io] is missing `{name}`"));
      assert_path_dependency(dep, tauri_path, crate_dir);
    }
  }

  #[test]
  fn version_dependencies() {
    let metadata = metadata();
    let data = tauri_dependencies_data(None, &metadata);
    assert!(
      !data.contains_key("patch_tauri_dep"),
      "patch_tauri_dep must only be set for path dependencies"
    );

    let manifest = render_manifest(&data);

    let tauri = dependency(&manifest, "dependencies", "tauri");
    assert_eq!(
      tauri.get("version").and_then(|v| v.as_str()),
      Some(metadata.tauri.as_str())
    );
    assert!(!tauri.contains_key("path"));

    let tauri_build = dependency(&manifest, "build-dependencies", "tauri-build");
    assert_eq!(
      tauri_build.get("version").and_then(|v| v.as_str()),
      Some(metadata.tauri_build.as_str())
    );
    assert!(!tauri_build.contains_key("path"));

    assert!(
      !manifest.contains_key("patch"),
      "[patch.crates-io] must not be rendered for version dependencies"
    );
  }

  #[test]
  fn relative_path_dependencies() {
    let tauri_path = Path::new("tauri-src");
    let manifest = render_manifest(&tauri_dependencies_data(Some(tauri_path), &metadata()));
    assert_manifest_uses_path(&manifest, tauri_path);
  }

  #[test]
  fn absolute_path_dependencies() {
    // absolute paths are written as-is; on Windows this covers backslash escaping
    let tauri_path = std::env::current_dir().unwrap().join("tauri-src");
    assert!(tauri_path.is_absolute());
    let manifest = render_manifest(&tauri_dependencies_data(Some(&tauri_path), &metadata()));
    assert_manifest_uses_path(&manifest, &tauri_path);
  }
}
