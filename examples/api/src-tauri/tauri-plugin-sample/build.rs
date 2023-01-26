use std::{
  env::var,
  fs,
  path::{PathBuf, MAIN_SEPARATOR},
};

fn main() {
  let manifest_dir = var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
  if let (Ok(out_dir), Ok(gradle_settings_path), Ok(app_build_gradle_path)) = (
    var("TAURI_PLUGIN_OUTPUT_PATH"),
    var("TAURI_GRADLE_SETTINGS_PATH"),
    var("TAURI_APP_GRADLE_BUILD_PATH"),
  ) {
    let source = manifest_dir.join("android");
    let pkg_name = var("CARGO_PKG_NAME").unwrap();

    println!("cargo:rerun-if-env-changed=TAURI_PLUGIN_OUTPUT_PATH");
    println!("cargo:rerun-if-env-changed=TAURI_GRADLE_SETTINGS_PATH");
    println!(
      "cargo:rerun-if-changed={}{}{}",
      out_dir, MAIN_SEPARATOR, pkg_name
    );

    let target = PathBuf::from(out_dir).join(&pkg_name);
    let _ = fs::remove_dir_all(&target);

    for entry in walkdir::WalkDir::new(&source) {
      let entry = entry.unwrap();
      let rel_path = entry.path().strip_prefix(&source).unwrap();
      let dest_path = target.join(rel_path);
      if entry.file_type().is_dir() {
        fs::create_dir(dest_path).expect("failed to create directory");
      } else {
        fs::copy(entry.path(), dest_path).expect("failed to copy Android template file");
      }
    }

    let gradle_settings = fs::read_to_string(&gradle_settings_path).unwrap();
    let plugin_name = "tauri-plugin-sample";
    let include = format!(
      "include ':{plugin_name}'
project(':{plugin_name}').projectDir = new File('./tauri-plugins/{plugin_name}')"
    );
    if !gradle_settings.contains(&include) {
      fs::write(
        &gradle_settings_path,
        &format!("{gradle_settings}\n{include}"),
      )
      .unwrap();
    }

    let app_build_gradle = fs::read_to_string(&app_build_gradle_path).unwrap();
    let implementation = format!(r#"implementation(project(":{plugin_name}"))"#);
    let target_implementation = r#"implementation(project(":tauri-android"))"#;
    if !app_build_gradle.contains(&implementation) {
      fs::write(
        &app_build_gradle_path,
        app_build_gradle.replace(
          &target_implementation,
          &format!("{target_implementation}\n    {implementation}"),
        ),
      )
      .unwrap();
    }
  }
}
