// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::AsShoutySnakeCase;

use std::env::var_os;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::{
  env::var,
  path::{Path, PathBuf},
  sync::{Mutex, OnceLock},
};

static CHECKED_FEATURES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

// checks if the given Cargo feature is enabled.
fn has_feature(feature: &str) -> bool {
  CHECKED_FEATURES
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .push(feature.to_string());

  // when a feature is enabled, Cargo sets the `CARGO_FEATURE_<name>` env var to 1
  // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
  std::env::var(format!("CARGO_FEATURE_{}", AsShoutySnakeCase(feature)))
    .map(|x| x == "1")
    .unwrap_or(false)
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
  if has_feature {
    println!("cargo:rustc-cfg={alias}");
  }
}

fn main() {
  alias("custom_protocol", has_feature("custom-protocol"));
  alias("dev", !has_feature("custom-protocol"));

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  alias("desktop", !mobile);
  alias("mobile", mobile);

  alias(
    "ipc_custom_protocol",
    target_os != "android" && (target_os != "linux" || has_feature("linux-ipc-protocol")),
  );

  let checked_features_out_path = Path::new(&var("OUT_DIR").unwrap()).join("checked_features");
  std::fs::write(
    checked_features_out_path,
    CHECKED_FEATURES.get().unwrap().lock().unwrap().join(","),
  )
  .expect("failed to write checked_features file");

  // workaround needed to prevent `STATUS_ENTRYPOINT_NOT_FOUND` error
  // see https://github.com/tauri-apps/tauri/pull/4383#issuecomment-1212221864
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
  let is_tauri_workspace = std::env::var("__TAURI_WORKSPACE__").map_or(false, |v| v == "true");
  if is_tauri_workspace && target_os == "windows" && Ok("msvc") == target_env.as_deref() {
    add_manifest();
  }

  if target_os == "android" {
    if let Ok(kotlin_out_dir) = std::env::var("WRY_ANDROID_KOTLIN_FILES_OUT_DIR") {
      fn env_var(var: &str) -> String {
        std::env::var(var).unwrap_or_else(|_| {
          panic!(
            "`{}` is not set, which is needed to generate the kotlin files for android.",
            var
          )
        })
      }

      let package = env_var("WRY_ANDROID_PACKAGE");
      let library = env_var("WRY_ANDROID_LIBRARY");

      let kotlin_out_dir = PathBuf::from(&kotlin_out_dir)
        .canonicalize()
        .unwrap_or_else(move |_| {
          panic!("Failed to canonicalize `WRY_ANDROID_KOTLIN_FILES_OUT_DIR` path {kotlin_out_dir}")
        });

      let kotlin_files_path =
        PathBuf::from(env_var("CARGO_MANIFEST_DIR")).join("mobile/android-codegen");
      println!("cargo:rerun-if-changed={}", kotlin_files_path.display());
      let kotlin_files =
        read_dir(kotlin_files_path).expect("failed to read Android codegen directory");

      for file in kotlin_files {
        let file = file.unwrap();

        let content = read_to_string(file.path())
          .expect("failed to read kotlin file as string")
          .replace("{{package}}", &package)
          .replace("{{library}}", &library);

        let out_path = kotlin_out_dir.join(file.file_name());
        write(&out_path, content).expect("Failed to write kotlin file");
        println!("cargo:rerun-if-changed={}", out_path.display());
      }
    }

    if let Some(project_dir) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
      let tauri_proguard = include_str!("./mobile/proguard-tauri.pro").replace(
        "$PACKAGE",
        &var("WRY_ANDROID_PACKAGE").expect("missing `WRY_ANDROID_PACKAGE` environment variable"),
      );
      std::fs::write(
        project_dir.join("app").join("proguard-tauri.pro"),
        tauri_proguard,
      )
      .expect("failed to write proguard-tauri.pro");
    }

    let lib_path =
      PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("mobile/android");
    println!("cargo:android_library_path={}", lib_path.display());
  }

  #[cfg(target_os = "macos")]
  {
    if target_os == "ios" {
      let lib_path =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("mobile/ios-api");
      tauri_build::mobile::link_swift_library("Tauri", &lib_path);
      println!("cargo:ios_library_path={}", lib_path.display());
    }
  }
}

fn add_manifest() {
  static WINDOWS_MANIFEST_FILE: &str = "window-app-manifest.xml";

  let manifest = std::env::current_dir()
    .unwrap()
    .join("../tauri-build/src")
    .join(WINDOWS_MANIFEST_FILE);

  println!("cargo:rerun-if-changed={}", manifest.display());
  // Embed the Windows application manifest file.
  println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
  println!(
    "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
    manifest.to_str().unwrap()
  );
  // Turn linker warnings into errors.
  println!("cargo:rustc-link-arg=/WX");
}
