// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  let mut context = tauri::generate_context!();
  if std::env::var("TARGET").unwrap_or_default() == "nsis" {
    context.config_mut().tauri.updater.windows.installer_args = vec![format!(
      "/D={}",
      std::env::current_exe().unwrap().parent().unwrap().display()
    )];
  }
  tauri::Builder::default()
    .setup(|app| {
      let handle = app.handle();
      tauri::async_runtime::spawn(async move {
        match handle.updater().check().await {
          Ok(update) => {
            if let Err(e) = update.download_and_install().await {
              println!("{e}");
              std::process::exit(1);
            }
            std::process::exit(0);
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
