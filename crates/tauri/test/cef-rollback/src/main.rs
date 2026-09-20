// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The application `tests/rollback.rs` drives.
//!
//! It runs on the CEF runtime with the root cache path and downgrade policy the test
//! names in its environment, loads a page that reads and bumps a launch counter kept
//! in the profile (`dist/index.html`), writes what the page found to the report file
//! and exits. Whether the counter survived a launch is how the test tells a kept
//! profile from a reset one.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tauri_runtime_cef::{Cef, DowngradePolicy, SandboxPolicy, SecretStorage};

/// Chromium's user data directory, `Cef::root_cache_path`. Required.
pub const ROOT_CACHE_PATH: &str = "CEF_ROLLBACK_ROOT_CACHE_PATH";
/// `keep` or `reset`; unset leaves the runtime's default.
pub const DOWNGRADE: &str = "CEF_ROLLBACK_DOWNGRADE";
/// The file the report is written to. Required.
pub const REPORT: &str = "CEF_ROLLBACK_REPORT";
/// When set, the app stays running after reporting, until it is killed.
pub const HOLD: &str = "CEF_ROLLBACK_HOLD";

/// What the page found in the profile on this launch.
#[derive(Serialize)]
struct Report {
  /// The Chromium this binary embeds, as `tauri_runtime_cef::webview_version` reports it.
  chromium_version: String,
  /// The launch counter before this launch bumped it.
  previous_runs: u32,
  /// Whether the cookie every launch sets was there.
  cookie_seen: bool,
}

#[tauri::command]
fn report(app: AppHandle, previous_runs: u32, cookie_seen: bool) -> Result<(), String> {
  let report = Report {
    chromium_version: tauri_runtime_cef::webview_version().map_err(|error| error.to_string())?,
    previous_runs,
    cookie_seen,
  };
  let path = PathBuf::from(std::env::var_os(REPORT).ok_or_else(|| format!("{REPORT} is not set"))?);
  // Written whole and renamed into place, so the test never reads a partial report.
  let staging = path.with_extension("json.part");
  std::fs::write(
    &staging,
    serde_json::to_vec(&report).map_err(|error| error.to_string())?,
  )
  .map_err(|error| error.to_string())?;
  std::fs::rename(&staging, &path).map_err(|error| error.to_string())?;

  if std::env::var_os(HOLD).is_none() {
    app.exit(0);
  }
  Ok(())
}

#[tauri_runtime_cef::cef_entry_point]
fn main() {
  let root_cache_path = PathBuf::from(std::env::var_os(ROOT_CACHE_PATH).unwrap_or_else(|| {
    panic!("{ROOT_CACHE_PATH} is not set; this binary is driven by tests/rollback.rs")
  }));
  let mut cef = Cef::default()
    .root_cache_path(&root_cache_path)
    // The sandbox is not what this test is about, and a bare test executable cannot have
    // one everywhere it runs: not on Windows, and not on a Linux that restricts
    // unprivileged user namespaces (see `SandboxPolicy`).
    .sandbox(SandboxPolicy::Disabled)
    // No keychain or keyring prompt on a test runner.
    .secret_storage(SecretStorage::Mock);
  match std::env::var(DOWNGRADE).as_deref() {
    Ok("keep") => cef = cef.downgrade(DowngradePolicy::KeepProfile),
    Ok("reset") => cef = cef.downgrade(DowngradePolicy::ResetProfile),
    Ok(other) => panic!("{DOWNGRADE} must be `keep` or `reset`, got {other:?}"),
    Err(_) => {}
  }

  tauri::Builder::default()
    .runtime(cef)
    .invoke_handler(tauri::generate_handler![report])
    .setup(|app| {
      WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("CEF rollback test")
        .inner_size(480., 320.)
        .build()?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running the CEF rollback test application");
}
