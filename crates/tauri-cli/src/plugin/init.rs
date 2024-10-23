// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::PluginIosFramework;
use crate::helpers::prompts;
use crate::Result;
use crate::{
  helpers::{resolve_tauri_path, template},
  VersionMetadata,
};
use anyhow::Context;
use clap::Parser;
use handlebars::{to_json, Handlebars};
use heck::{ToKebabCase, ToPascalCase, ToSnakeCase};
use include_dir::{include_dir, Dir};
use std::ffi::{OsStr, OsString};
use std::{
  collections::BTreeMap,
  env::current_dir,
  fs::{create_dir_all, remove_dir_all, File, OpenOptions},
  path::{Component, Path, PathBuf},
};

pub const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/plugin");

#[derive(Debug, Parser)]
#[clap(about = "Initialize a Tauri plugin project on an existing directory")]
pub struct Options {
  /// Name of your Tauri plugin.
  /// If not specified, it will be inferred from the current directory.
  pub(crate) plugin_name: Option<String>,
  /// Initializes a Tauri plugin without the TypeScript API
  #[clap(long)]
  pub(crate) no_api: bool,
  /// Initialize without an example project.
  #[clap(long)]
  pub(crate) no_example: bool,
  /// Set target directory for init
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
  pub(crate) directory: String,
  /// Author name
  #[clap(short, long)]
  pub(crate) author: Option<String>,
  /// Whether to initialize an Android project for the plugin.
  #[clap(long)]
  pub(crate) android: bool,
  /// Whether to initialize an iOS project for the plugin.
  #[clap(long)]
  pub(crate) ios: bool,
  /// Whether to initialize Android and iOS projects for the plugin.
  #[clap(long)]
  pub(crate) mobile: bool,
  /// Type of framework to use for the iOS project.
  #[clap(long)]
  #[clap(default_value_t = PluginIosFramework::default())]
  pub(crate) ios_framework: PluginIosFramework,
  /// Generate github workflows
  #[clap(long)]
  pub(crate) github_workflows: bool,

  /// Initializes a Tauri core plugin (internal usage)
  #[clap(long, hide(true))]
  pub(crate) tauri: bool,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  pub(crate) tauri_path: Option<PathBuf>,
}

impl Options {
  fn load(&mut self) {
    if self.author.is_none() {
      self.author.replace(if self.tauri {
        "Tauri Programme within The Commons Conservancy".into()
      } else {
        "You".into()
      });
    }
  }
}

pub fn command(mut options: Options) -> Result<()> {
  options.load();

  let plugin_name = match options.plugin_name {
    None => super::infer_plugin_name(&options.directory)?,
    Some(name) => name,
  };

  let template_target_path = PathBuf::from(options.directory);
  let metadata = crates_metadata()?;
  if std::fs::read_dir(&template_target_path)?.count() > 0 {
    log::warn!("Plugin dir ({:?}) not empty.", template_target_path);
  } else {
    let (tauri_dep, tauri_example_dep, tauri_build_dep, tauri_plugin_dep) =
      if let Some(tauri_path) = options.tauri_path {
        (
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "crates/tauri")
          ),
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "crates/tauri")
          ),
          format!(
            "{{  path = {:?}, default-features = false }}",
            resolve_tauri_path(&tauri_path, "crates/tauri-build")
          ),
          format!(
            r#"{{  path = {:?}, features = ["build"] }}"#,
            resolve_tauri_path(&tauri_path, "crates/tauri-plugin")
          ),
        )
      } else {
        (
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(
            r#"{{ version = "{}", default-features = false }}"#,
            metadata.tauri_build
          ),
          format!(
            r#"{{ version = "{}", features = ["build"] }}"#,
            metadata.tauri_plugin
          ),
        )
      };

    let _ = remove_dir_all(&template_target_path);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let mut data = BTreeMap::new();
    plugin_name_data(&mut data, &plugin_name);
    data.insert("tauri_dep", to_json(tauri_dep));
    data.insert("tauri_example_dep", to_json(tauri_example_dep));
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert("tauri_plugin_dep", to_json(tauri_plugin_dep));
    data.insert("author", to_json(options.author));

    if options.tauri {
      data.insert(
        "license_header",
        to_json(
          "// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
             // SPDX-License-Identifier: Apache-2.0
             // SPDX-License-Identifier: MIT\n\n"
            .replace("  ", "")
            .replace(" //", "//"),
        ),
      );
    }

    let plugin_id = if options.android || options.mobile {
      let plugin_id = prompts::input(
        "What should be the Android Package ID for your plugin?",
        Some(format!("com.plugin.{}", plugin_name)),
        false,
        false,
      )?
      .unwrap();

      data.insert("android_package_id", to_json(&plugin_id));
      Some(plugin_id)
    } else {
      None
    };

    let ios_framework = options.ios_framework;

    let mut created_dirs = Vec::new();
    template::render_with_generator(
      &handlebars,
      &data,
      &TEMPLATE_DIR,
      &template_target_path,
      &mut |mut path| {
        let mut components = path.components();
        let root = components.next().unwrap();

        if let Component::Normal(component) = root {
          match component.to_str().unwrap() {
            "__example-api" => {
              if options.no_api || options.no_example {
                return Ok(None);
              } else {
                path = Path::new("examples").join(components.collect::<PathBuf>());
              }
            }
            "__example-basic" => {
              if options.no_api && !options.no_example {
                path = Path::new("examples").join(components.collect::<PathBuf>());
              } else {
                return Ok(None);
              }
            }
            ".github" if !options.github_workflows => return Ok(None),
            "android" => {
              if options.android || options.mobile {
                return generate_android_out_file(
                  &path,
                  &template_target_path,
                  &plugin_id.as_ref().unwrap().replace('.', "/"),
                  &mut created_dirs,
                );
              } else {
                return Ok(None);
              }
            }
            "ios-spm" | "ios-xcode" if !(options.ios || options.mobile) => return Ok(None),
            "ios-spm" if !matches!(ios_framework, PluginIosFramework::Spm) => return Ok(None),
            "ios-xcode" if !matches!(ios_framework, PluginIosFramework::Xcode) => return Ok(None),
            "ios-spm" | "ios-xcode" => {
              let folder_name = components.next().unwrap().as_os_str().to_string_lossy();
              let new_folder_name = folder_name.replace("{{ plugin_name }}", &plugin_name);
              let new_folder_name = OsString::from(&new_folder_name);

              path = [
                Component::Normal(OsStr::new("ios")),
                Component::Normal(&new_folder_name),
              ]
              .into_iter()
              .chain(components)
              .collect::<PathBuf>();
            }
            "guest-js" | "rollup.config.js" | "tsconfig.json" | "package.json"
              if options.no_api =>
            {
              return Ok(None);
            }
            _ => (),
          }
        }

        let path = template_target_path.join(path);
        let parent = path.parent().unwrap().to_path_buf();
        if !created_dirs.contains(&parent) {
          create_dir_all(&parent)?;
          created_dirs.push(parent);
        }
        File::create(path).map(Some)
      },
    )
    .with_context(|| "failed to render plugin template")?;
  }

  let permissions_dir = template_target_path.join("permissions");
  std::fs::create_dir(&permissions_dir)
    .with_context(|| "failed to create `permissions` directory")?;

  let default_permissions = r#"[default]
description = "Default permissions for the plugin"
permissions = ["allow-ping"]
"#;
  std::fs::write(permissions_dir.join("default.toml"), default_permissions)
    .with_context(|| "failed to write `permissions/default.toml`")?;

  Ok(())
}

pub fn plugin_name_data(data: &mut BTreeMap<&'static str, serde_json::Value>, plugin_name: &str) {
  data.insert("plugin_name_original", to_json(plugin_name));
  data.insert("plugin_name", to_json(plugin_name.to_kebab_case()));
  data.insert(
    "plugin_name_snake_case",
    to_json(plugin_name.to_snake_case()),
  );
  data.insert(
    "plugin_name_pascal_case",
    to_json(plugin_name.to_pascal_case()),
  );
}

pub fn crates_metadata() -> Result<VersionMetadata> {
  serde_json::from_str::<VersionMetadata>(include_str!("../../metadata-v2.json"))
    .map_err(Into::into)
}

pub fn generate_android_out_file(
  path: &Path,
  dest: &Path,
  package_path: &str,
  created_dirs: &mut Vec<PathBuf>,
) -> std::io::Result<Option<File>> {
  let mut iter = path.iter();
  let root = iter.next().unwrap().to_str().unwrap();
  let path = match (root, path.extension().and_then(|o| o.to_str())) {
    ("src", Some("kt")) => {
      let parent = path.parent().unwrap();
      let file_name = path.file_name().unwrap();
      let out_dir = dest.join(parent).join(package_path);
      out_dir.join(file_name)
    }
    _ => dest.join(path),
  };

  let parent = path.parent().unwrap().to_path_buf();
  if !created_dirs.contains(&parent) {
    create_dir_all(&parent)?;
    created_dirs.push(parent);
  }

  let mut options = OpenOptions::new();
  options.write(true);

  #[cfg(unix)]
  if path.file_name().unwrap() == std::ffi::OsStr::new("gradlew") {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o755);
  }

  if !path.exists() {
    options.create(true).open(path).map(Some)
  } else {
    Ok(None)
  }
}
