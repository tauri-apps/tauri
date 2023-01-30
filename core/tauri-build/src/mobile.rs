use std::{
  env::var,
  fs,
  path::{PathBuf, MAIN_SEPARATOR},
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
        if let (Ok(out_dir), Ok(gradle_settings_path), Ok(app_build_gradle_path)) = (
          var("TAURI_PLUGIN_OUTPUT_PATH"),
          var("TAURI_GRADLE_SETTINGS_PATH"),
          var("TAURI_APP_GRADLE_BUILD_PATH"),
        ) {
          let source = manifest_dir.join(path);
          let pkg_name = var("CARGO_PKG_NAME").unwrap();

          println!("cargo:rerun-if-env-changed=TAURI_PLUGIN_OUTPUT_PATH");
          println!("cargo:rerun-if-env-changed=TAURI_GRADLE_SETTINGS_PATH");
          println!(
            "cargo:rerun-if-changed={}{}{}",
            out_dir, MAIN_SEPARATOR, pkg_name
          );
          println!("cargo:rerun-if-changed={}", gradle_settings_path);
          println!("cargo:rerun-if-changed={}", app_build_gradle_path);

          let target = PathBuf::from(out_dir).join(&pkg_name);
          let _ = fs::remove_dir_all(&target);

          for entry in walkdir::WalkDir::new(&source) {
            let entry = entry?;
            let rel_path = entry.path().strip_prefix(&source)?;
            let dest_path = target.join(rel_path);
            if entry.file_type().is_dir() {
              fs::create_dir(dest_path)?;
            } else {
              fs::copy(entry.path(), dest_path)?;
            }
          }

          let gradle_settings = fs::read_to_string(&gradle_settings_path)?;
          let include = format!(
            "include ':{pkg_name}'
project(':{pkg_name}').projectDir = new File('./tauri-plugins/{pkg_name}')"
          );
          if !gradle_settings.contains(&include) {
            fs::write(
              &gradle_settings_path,
              &format!("{gradle_settings}\n{include}"),
            )?;
          }

          let app_build_gradle = fs::read_to_string(&app_build_gradle_path)?;
          let implementation = format!(r#"implementation(project(":{pkg_name}"))"#);
          let target_implementation = r#"implementation(project(":tauri-android"))"#;
          if !app_build_gradle.contains(&implementation) {
            fs::write(
              &app_build_gradle_path,
              app_build_gradle.replace(
                target_implementation,
                &format!("{target_implementation}\n    {implementation}"),
              ),
            )?
          }
        }
      }
    }

    Ok(())
  }
}
