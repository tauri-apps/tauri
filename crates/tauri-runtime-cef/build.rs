// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  if target_os == "android" {
    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let android = manifest_dir.join("src/android");
    println!(
      "cargo:android_app_gradle_path={}",
      android.join("cef.gradle.kts").display()
    );
    println!(
      "cargo:android_codegen_path={}",
      android.join("TauriActivity.kt").display()
    );
    println!(
      "cargo:java_android_codegen_path={}",
      android.join("CefActivity.java").display()
    );

    println!("cargo:rerun-if-env-changed=TAURI_ANDROID_PROJECT_PATH");
    println!("cargo:rerun-if-env-changed=TAURI_ANDROID_LIBRARY");

    if let Ok(library) = std::env::var("TAURI_ANDROID_LIBRARY") {
      tauri_utils::build::update_android_manifest(
        "tauri-cef-game-activity",
        "activity",
        format!(r#"<meta-data android:name="android.app.lib_name" android:value="{library}" />"#),
      )
      .expect("failed to add GameActivity library metadata");
    }
  }

  println!("cargo:rerun-if-changed=build.rs");
  // exposed to the build scripts of crates depending on this one as `DEP_TAURI_RUNTIME_CEF_RUNTIME`,
  // which is how `tauri-build` detects that the application uses the CEF runtime.
  println!("cargo:runtime=cef");
  // The CEF binary distribution `cef-dll-sys` resolved for this build, downloading it when it
  // was missing, passed on to the build scripts of crates depending on this one as
  // `DEP_TAURI_RUNTIME_CEF_CEF_DIR`: where the framework, `libcef` and the resources are.
  println!("cargo:rerun-if-env-changed=DEP_CEF_DLL_WRAPPER_CEF_DIR");
  if let Ok(cef_dir) = std::env::var("DEP_CEF_DLL_WRAPPER_CEF_DIR") {
    println!("cargo:cef_dir={cef_dir}");
  }
}
