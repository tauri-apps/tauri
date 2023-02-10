use std::{
  env::var,
  fs::{self, rename},
  path::{Path, PathBuf},
};

use anyhow::Result;

#[derive(Default)]
pub struct PluginBuilder {
  android_path: Option<PathBuf>,
}

impl PluginBuilder {
  /// Creates a new builder for mobile plugin functionality.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the Android project path.
  pub fn android_path<P: Into<PathBuf>>(mut self, android_path: P) -> Self {
    self.android_path.replace(android_path.into());
    self
  }

  /// Injects the mobile templates in the given path relative to the manifest root.
  pub fn run(self) -> Result<()> {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "android" {
      if let Some(path) = self.android_path {
        let manifest_dir = var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
        if let Ok(project_dir) = var("TAURI_ANDROID_PROJECT_PATH") {
          let source = manifest_dir.join(path);
          let pkg_name = var("CARGO_PKG_NAME").unwrap();

          println!("cargo:rerun-if-env-changed=TAURI_ANDROID_PROJECT_PATH");

          let project_dir = PathBuf::from(project_dir);

          inject_android_project(source, project_dir.join("tauri-plugins").join(&pkg_name))?;

          let gradle_settings_path = project_dir.join("tauri.settings.gradle");
          let gradle_settings = fs::read_to_string(&gradle_settings_path)?;
          let include = format!(
            "include ':{pkg_name}'
project(':{pkg_name}').projectDir = new File('./tauri-plugins/{pkg_name}')"
          );
          if !gradle_settings.contains(&include) {
            fs::write(
              &gradle_settings_path,
              format!("{gradle_settings}\n{include}"),
            )?;
          }

          let app_build_gradle_path = project_dir.join("app").join("tauri.build.gradle.kts");
          let app_build_gradle = fs::read_to_string(&app_build_gradle_path)?;
          let implementation = format!(r#"implementation(project(":{pkg_name}"))"#);
          let target = "dependencies {";
          if !app_build_gradle.contains(&implementation) {
            fs::write(
              &app_build_gradle_path,
              app_build_gradle.replace(target, &format!("{target}\n  {implementation}")),
            )?
          }
        }
      }
    }

    Ok(())
  }
}

#[doc(hidden)]
pub fn inject_android_project(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
  let source = source.as_ref();
  let target = target.as_ref();

  // keep build folder if it exists
  let build_path = target.join("build");
  let out_dir = if build_path.exists() {
    let out_dir = target.parent().unwrap().join(".tauri-tmp-build");
    rename(&build_path, &out_dir)?;
    Some(out_dir)
  } else {
    None
  };

  let _ = fs::remove_dir_all(target);

  for entry in walkdir::WalkDir::new(source) {
    let entry = entry?;
    let rel_path = entry.path().strip_prefix(source)?;
    let dest_path = target.join(rel_path);
    if entry.file_type().is_dir() {
      fs::create_dir(dest_path)?;
    } else {
      fs::copy(entry.path(), dest_path)?;
    }
  }

  if let Some(out_dir) = out_dir {
    rename(out_dir, &build_path)?;
  }

  Ok(())
}
