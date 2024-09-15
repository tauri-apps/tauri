// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

fn main() {
  eprintln!("running tauri v1 app...");
  let mut context = tauri::generate_context!();
  if std::env::var("TARGET").unwrap_or_default() == "nsis" {
    // /D sets the default installation directory ($INSTDIR),
    // overriding InstallDir and InstallDirRegKey.
    // It must be the last parameter used in the command line and must not contain any quotes, even if the path contains spaces.
    // Only absolute paths are supported.
    // NOTE: we only need this because this is an integration test and we don't want to install the app in the programs folder
    context.config_mut().tauri.updater.windows.installer_args = vec![format!(
      "/D={}",
      tauri::utils::platform::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .display()
    )];
  }
  tauri::Builder::default()
    .setup(|app| {
      println!("current version: {}", app.package_info().version);
      let handle = app.handle();
      tauri::async_runtime::spawn(async move {
        match handle
          .updater()
          .timeout(Duration::from_secs(1))
          .check()
          .await
        {
          Ok(update) => {
            println!("got update {}", update.latest_version());
            if update.is_update_available() {
              if let Err(e) = update.download_and_install().await {
                println!("{e}");
                std::process::exit(1);
              }
              std::process::exit(0);
            } else {
              std::process::exit(2);
            }
          }
          Err(e) => {
            println!("{e}");
            std::process::exit(1);
          }
        }
      });
      Ok(())
    })
    .run(context)
    .expect("error while running tauri application");
}
