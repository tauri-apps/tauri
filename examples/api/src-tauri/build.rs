// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_build::WindowsAttributes;

fn main() {
  tauri_build::try_build(
    tauri_build::Attributes::new()
      .codegen(tauri_build::CodegenContext::new())
      .windows_attributes(WindowsAttributes::new_without_app_manifest())
      .plugin(
        "app-menu",
        tauri_build::InlinedPlugin::new().commands(&["toggle", "popup"]),
      )
      .app_manifest(tauri_build::AppManifest::new().commands(&[
        "log_operation",
        "perform_request",
        "echo",
      ])),
  )
  .expect("failed to run tauri-build");

  // workaround needed to prevent `STATUS_ENTRYPOINT_NOT_FOUND` error in tests
  // see https://github.com/tauri-apps/tauri/pull/4383#issuecomment-1212221864
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
  let is_tauri_workspace = std::env::var("__TAURI_WORKSPACE__").map_or(false, |v| v == "true");
  if is_tauri_workspace && target_os == "windows" && Ok("msvc") == target_env.as_deref() {
    embed_manifest_for_tests();
  }
}

fn embed_manifest_for_tests() {
  static WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

  let manifest = std::env::current_dir()
    .unwrap()
    .join("../../../crates/tauri-build/src")
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
