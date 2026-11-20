// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `Builder::try_build` reads the working directory and environment Cargo gives a build script,
//! which a test can only set for the whole process, so this binary holds a single test.

use std::fs;

#[test]
fn command_less_plugin_leaves_read_only_source_untouched() {
  let plugin_dir = tempfile::tempdir().unwrap();
  let out_dir = tempfile::tempdir().unwrap();

  // SAFETY: this is the only test of the binary, so no other thread reads the environment
  unsafe {
    std::env::set_var("CARGO_PKG_NAME", "tauri-plugin-fixture");
    std::env::set_var("CARGO_MANIFEST_LINKS", "tauri-plugin-fixture");
    std::env::set_var("CARGO_CFG_TARGET_OS", "linux");
    std::env::set_var("OUT_DIR", out_dir.path());
  }

  let writable = fs::metadata(plugin_dir.path()).unwrap().permissions();
  let mut read_only = writable.clone();
  read_only.set_readonly(true);
  fs::set_permissions(plugin_dir.path(), read_only).unwrap();
  let cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(plugin_dir.path()).unwrap();

  let result = std::panic::catch_unwind(|| tauri_plugin::Builder::new(&[]).try_build());

  std::env::set_current_dir(cwd).unwrap();
  fs::set_permissions(plugin_dir.path(), writable).unwrap();
  result.unwrap().unwrap();
  let created = fs::read_dir(plugin_dir.path())
    .unwrap()
    .map(|entry| entry.unwrap().file_name())
    .collect::<Vec<_>>();
  assert!(
    created.is_empty(),
    "build wrote {created:?} into the plugin directory"
  );
}
