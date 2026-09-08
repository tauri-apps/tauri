// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn init_force_skips_prompts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();

    // Determine the path to the cargo-tauri binary.
    // The test binary is in target/debug/deps/, so we navigate up to target/debug/
    let current_exe = std::env::current_exe().unwrap();
    let target_debug = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let cargo_tauri = if cfg!(windows) {
        target_debug.join("cargo-tauri.exe")
    } else {
        target_debug.join("cargo-tauri")
    };

    assert!(cargo_tauri.exists(), "cargo-tauri binary not found at {:?}", cargo_tauri);

    // Run `init --force` with some explicit options.
    let output = Command::new(&cargo_tauri)
        .arg("init")
        .arg("--force")
        .arg("--app-name")
        .arg("testapp")
        .arg("--frontend-dist")
        .arg("../dist")
        .arg("--dev-url")
        .arg("http://localhost:3000")
        .current_dir(dir)
        .output()
        .expect("failed to run cargo-tauri init");

    assert!(output.status.success(), "init failed: {:?}", output);

    // Verify that src-tauri was created.
    let src_tauri = dir.join("src-tauri");
    assert!(src_tauri.exists(), "src-tauri directory was not created");

    // Verify that the tauri.conf.json contains the app name we set.
    let conf_path = src_tauri.join("tauri.conf.json");
    assert!(conf_path.exists(), "tauri.conf.json not found");
    let content = fs::read_to_string(&conf_path).unwrap();
    assert!(content.contains("testapp"), "app name not found in config");
}
