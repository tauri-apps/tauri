// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::AsShoutySnakeCase;
use heck::AsSnakeCase;
use heck::ToSnakeCase;

use once_cell::sync::OnceCell;

use std::env::var_os;
use std::{
  env::var,
  path::{Path, PathBuf},
  sync::Mutex,
};

static CHECKED_FEATURES: OnceCell<Mutex<Vec<String>>> = OnceCell::new();

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
  alias("updater", has_feature("updater"));

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  alias("desktop", !mobile);
  alias("mobile", mobile);

  let api_all = has_feature("api-all");
  alias("api_all", api_all);

  alias_module(
    "fs",
    &[
      "read-file",
      "write-file",
      "read-dir",
      "copy-file",
      "create-dir",
      "remove-dir",
      "remove-file",
      "rename-file",
      "exists",
    ],
    api_all,
  );

  alias_module(
    "window",
    &[
      "create",
      "center",
      "request-user-attention",
      "set-resizable",
      "set-title",
      "maximize",
      "unmaximize",
      "minimize",
      "unminimize",
      "show",
      "hide",
      "close",
      "set-decorations",
      "set-shadow",
      "set-always-on-top",
      "set-content-protected",
      "set-size",
      "set-min-size",
      "set-max-size",
      "set-position",
      "set-fullscreen",
      "set-focus",
      "set-icon",
      "set-skip-taskbar",
      "set-cursor-grab",
      "set-cursor-visible",
      "set-cursor-icon",
      "set-cursor-position",
      "set-ignore-cursor-events",
      "start-dragging",
      "print",
    ],
    api_all,
  );

  alias_module("shell", &["execute", "sidecar", "open"], api_all);
  // helper for the command module macro
  let shell_script = has_feature("shell-execute") || has_feature("shell-sidecar");
  alias("shell_script", shell_script);
  alias("shell_scope", has_feature("shell-open-api") || shell_script);

  if !mobile {
    alias_module(
      "dialog",
      &["open", "save", "message", "ask", "confirm"],
      api_all,
    );
  }

  alias_module("http", &["request"], api_all);

  if !mobile {
    alias_module("notification", &[], api_all);
    alias_module("global-shortcut", &[], api_all);
  }
  alias_module("os", &[], api_all);
  alias_module("path", &[], api_all);

  alias_module("protocol", &["asset"], api_all);

  alias_module("process", &["relaunch", "exit"], api_all);

  alias_module("clipboard", &["write-text", "read-text"], api_all);

  alias_module("app", &["show", "hide"], api_all);

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
    if let Some(project_dir) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
      tauri_build::mobile::inject_android_project(
        "./mobile/android",
        project_dir.join(".tauri").join("tauri-api"),
        &[],
      )
      .expect("failed to copy tauri-api Android project");
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

// create aliases for the given module with its apis.
// each api is translated into a feature flag in the format of `<module>-<api>`
// and aliased as `<module_snake_case>_<api_snake_case>`.
//
// The `<module>-all` feature is also aliased to `<module>_all`.
//
// If any of the features is enabled, the `<module_snake_case>_any` alias is created.
//
// Note that both `module` and `apis` strings must be written in kebab case.
fn alias_module(module: &str, apis: &[&str], api_all: bool) {
  let all_feature_name = format!("{module}-all");
  let all = has_feature(&all_feature_name) || api_all;
  alias(&all_feature_name.to_snake_case(), all);

  let mut any = all;

  for api in apis {
    let has = has_feature(&format!("{module}-{api}")) || all;
    alias(
      &format!("{}_{}", AsSnakeCase(module), AsSnakeCase(api)),
      has,
    );
    any = any || has;
  }

  alias(&format!("{}_any", AsSnakeCase(module)), any);
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
