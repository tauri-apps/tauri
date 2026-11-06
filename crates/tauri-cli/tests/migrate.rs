// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fs;
use std::process::Command;

#[test]
fn migrate_v1_with_major_only_version_requirement() {
  let temp_dir = tempfile::tempdir().unwrap();
  let dir = temp_dir.path();

  let current_exe = std::env::current_exe().unwrap();
  let target_debug = current_exe.parent().unwrap().parent().unwrap();
  let cargo_tauri = if cfg!(windows) {
    target_debug.join("cargo-tauri.exe")
  } else {
    target_debug.join("cargo-tauri")
  };

  assert!(
    cargo_tauri.exists(),
    "cargo-tauri binary not found at {:?}",
    cargo_tauri
  );

  let src_tauri = dir.join("src-tauri");
  fs::create_dir_all(src_tauri.join("src")).unwrap();
  fs::create_dir_all(dir.join("dist")).unwrap();
  fs::write(
    dir.join("package.json"),
    r#"{"name":"app","version":"0.0.0"}"#,
  )
  .unwrap();
  fs::write(src_tauri.join("src/main.rs"), "fn main() {}").unwrap();
  fs::write(
    src_tauri.join("Cargo.toml"),
    r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1", features = [] }

[dependencies]
tauri = { version = "1", features = [] }
"#,
  )
  .unwrap();
  fs::write(
    src_tauri.join("tauri.conf.json"),
    r#"{
  "build": { "distDir": "../dist", "devPath": "../dist" },
  "package": { "productName": "app", "version": "0.1.0" },
  "tauri": {
    "bundle": { "identifier": "com.example.app" },
    "windows": [{ "title": "app" }]
  }
}"#,
  )
  .unwrap();

  let output = Command::new(&cargo_tauri)
    .arg("migrate")
    .current_dir(dir)
    .output()
    .expect("failed to run cargo-tauri migrate");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    output.status.success() && !stderr.contains("failed to parse tauri version"),
    "migrate failed: {output:?}"
  );

  let manifest = fs::read_to_string(src_tauri.join("Cargo.toml")).unwrap();
  assert!(
    manifest.contains(r#"tauri = { version = "2""#),
    "tauri dependency was not migrated: {manifest}"
  );
}
