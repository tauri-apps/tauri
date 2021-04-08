use std::{
  collections::BTreeMap,
  fs::{create_dir_all, remove_dir_all, File},
  io::Write,
  path::{Path, PathBuf},
};

use crate::helpers::Logger;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};
use serde::Deserialize;

const TEMPLATE_DIR: Dir = include_dir!("./templates");

#[derive(Deserialize)]
struct ManifestPackage {
  version: String,
}

#[derive(Deserialize)]
struct Manifest {
  package: ManifestPackage,
}

pub struct Init {
  force: bool,
  directory: PathBuf,
  tauri_path: Option<PathBuf>,
  app_name: Option<String>,
  window_title: Option<String>,
  dist_dir: Option<String>,
  dev_path: Option<String>,
}

impl Default for Init {
  fn default() -> Self {
    Self {
      force: false,
      directory: std::env::current_dir().expect("failed to read cwd"),
      tauri_path: None,
      app_name: None,
      window_title: None,
      dist_dir: None,
      dev_path: None,
    }
  }
}

impl Init {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn force(mut self) -> Self {
    self.force = true;
    self
  }

  pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
    self.directory = directory.into();
    self
  }

  pub fn tauri_path(mut self, tauri_path: impl Into<PathBuf>) -> Self {
    self.tauri_path = Some(tauri_path.into());
    self
  }

  pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
    self.app_name = Some(app_name.into());
    self
  }

  pub fn window_title(mut self, window_title: impl Into<String>) -> Self {
    self.window_title = Some(window_title.into());
    self
  }

  pub fn dist_dir(mut self, dist_dir: impl Into<String>) -> Self {
    self.dist_dir = Some(dist_dir.into());
    self
  }

  pub fn dev_path(mut self, dev_path: impl Into<String>) -> Self {
    self.dev_path = Some(dev_path.into());
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let logger = Logger::new("tauri:init");
    let template_target_path = self.directory.join("src-tauri");
    if template_target_path.exists() && !self.force {
      logger.warn(format!(
        "Tauri dir ({:?}) not empty. Run `init --force template` to overwrite.",
        template_target_path
      ));
    } else {
      let (tauri_dep, tauri_build_dep) = if let Some(tauri_path) = self.tauri_path {
        (
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri-build")
          ),
        )
      } else {
        let tauri_manifest: Manifest =
          toml::from_str(include_str!("../../../core/tauri/Cargo.toml")).unwrap();
        let tauri_build_manifest: Manifest =
          toml::from_str(include_str!("../../../core/tauri-build/Cargo.toml")).unwrap();
        (
          format!(r#"{{ version = "{}" }}"#, tauri_manifest.package.version),
          format!(
            r#"{{ version = "{}" }}"#,
            tauri_build_manifest.package.version
          ),
        )
      };

      let _ = remove_dir_all(&template_target_path);
      let handlebars = Handlebars::new();

      let mut data = BTreeMap::new();
      data.insert("tauri_dep", to_json(tauri_dep));
      data.insert("tauri_build_dep", to_json(tauri_build_dep));
      data.insert(
        "dist_dir",
        to_json(self.dist_dir.unwrap_or_else(|| "../dist".to_string())),
      );
      data.insert(
        "dev_path",
        to_json(
          self
            .dev_path
            .unwrap_or_else(|| "http://localhost:4000".to_string()),
        ),
      );
      data.insert(
        "app_name",
        to_json(self.app_name.unwrap_or_else(|| "Tauri App".to_string())),
      );
      data.insert(
        "window_title",
        to_json(self.window_title.unwrap_or_else(|| "Tauri".to_string())),
      );

      render_template(&handlebars, &data, &TEMPLATE_DIR, &self.directory)?;
    }

    Ok(())
  }
}

fn render_template<P: AsRef<Path>>(
  handlebars: &Handlebars,
  data: &BTreeMap<&str, serde_json::Value>,
  dir: &Dir,
  out_dir: P,
) -> crate::Result<()> {
  create_dir_all(out_dir.as_ref().join(dir.path()))?;
  for file in dir.files() {
    let mut output_file = File::create(out_dir.as_ref().join(file.path()))?;
    if let Some(utf8) = file.contents_utf8() {
      handlebars
        .render_template_to_write(utf8, &data, &mut output_file)
        .expect("Failed to render template");
    } else {
      output_file.write_all(file.contents())?;
    }
  }
  for dir in dir.dirs() {
    render_template(handlebars, data, dir, out_dir.as_ref())?;
  }
  Ok(())
}

fn resolve_tauri_path<P: AsRef<Path>>(path: P, crate_name: &str) -> PathBuf {
  let path = path.as_ref();
  if path.is_absolute() {
    path.join(crate_name)
  } else {
    PathBuf::from("..").join(path).join(crate_name)
  }
}
