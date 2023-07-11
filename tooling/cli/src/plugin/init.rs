// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use crate::{
  helpers::{resolve_tauri_path, template},
  VersionMetadata,
};
use anyhow::Context;
use clap::Parser;
use dialoguer::Input;
use handlebars::{to_json, Handlebars};
use heck::{AsKebabCase, ToKebabCase, ToPascalCase, ToSnakeCase};
use include_dir::{include_dir, Dir};
use log::warn;
use std::{
  collections::BTreeMap,
  env::current_dir,
  ffi::OsStr,
  fmt::Display,
  fs::{create_dir_all, remove_dir_all, File, OpenOptions},
  path::{Component, Path, PathBuf},
  str::FromStr,
};

pub const TEMPLATE_DIR: Dir<'_> = include_dir!("templates/plugin");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri plugin project")]
pub struct Options {
  /// Name of your Tauri plugin
  #[clap(short = 'n', long = "name")]
  plugin_name: String,
  /// Initializes a Tauri plugin without the TypeScript API
  #[clap(long)]
  no_api: bool,
  /// Initializes a Tauri core plugin (internal usage)
  #[clap(long, hide(true))]
  tauri: bool,
  /// Set target directory for init
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
  directory: String,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  tauri_path: Option<PathBuf>,
  /// Author name
  #[clap(short, long)]
  author: Option<String>,
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
  let template_target_path = PathBuf::from(options.directory).join(format!(
    "tauri-plugin-{}",
    AsKebabCase(&options.plugin_name)
  ));
  let metadata = crates_metadata()?;
  if template_target_path.exists() {
    warn!("Plugin dir ({:?}) not empty.", template_target_path);
  } else {
    let (tauri_dep, tauri_example_dep, tauri_build_dep) =
      if let Some(tauri_path) = options.tauri_path {
        (
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri-build")
          ),
        )
      } else {
        (
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
        )
      };

    let _ = remove_dir_all(&template_target_path);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let mut data = BTreeMap::new();
    plugin_name_data(&mut data, &options.plugin_name);
    data.insert("tauri_dep", to_json(tauri_dep));
    data.insert("tauri_example_dep", to_json(tauri_example_dep));
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert("author", to_json(options.author));

    if options.tauri {
      data.insert(
        "license_header",
        to_json(
          "// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
             // SPDX-License-Identifier: Apache-2.0
             // SPDX-License-Identifier: MIT\n\n"
            .replace("  ", "")
            .replace(" //", "//"),
        ),
      );
    }

    let plugin_id = request_input(
      "What should be the Android Package ID for your plugin?",
      Some(format!("com.plugin.{}", options.plugin_name)),
      false,
      false,
    )?
    .unwrap();

    data.insert("android_package_id", to_json(&plugin_id));

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
              if options.no_api {
                return Ok(None);
              } else {
                path = Path::new("examples").join(components.collect::<PathBuf>());
              }
            }
            "__example-basic" => {
              if options.no_api {
                path = Path::new("examples").join(components.collect::<PathBuf>());
              } else {
                return Ok(None);
              }
            }
            "android" => {
              return generate_android_out_file(
                &path,
                &template_target_path,
                &plugin_id.replace('.', "/"),
                &mut created_dirs,
              );
            }
            "webview-dist" | "webview-src" | "package.json" => {
              if options.no_api {
                return Ok(None);
              }
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
    .with_context(|| "failed to render plugin Android template")?;
  }
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
  if path.file_name().unwrap() == OsStr::new("gradlew") {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o755);
  }

  if path.file_name().unwrap() == OsStr::new("BuildTask.kt") || !path.exists() {
    options.create(true).open(path).map(Some)
  } else {
    Ok(None)
  }
}

pub fn request_input<T>(
  prompt: &str,
  initial: Option<T>,
  skip: bool,
  allow_empty: bool,
) -> Result<Option<T>>
where
  T: Clone + FromStr + Display + ToString,
  T::Err: Display + std::fmt::Debug,
{
  if skip {
    Ok(initial)
  } else {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut builder = Input::with_theme(&theme);
    builder.with_prompt(prompt);
    builder.allow_empty(allow_empty);

    if let Some(v) = initial {
      builder.with_initial_text(v.to_string());
    }

    builder.interact_text().map(Some).map_err(Into::into)
  }
}
