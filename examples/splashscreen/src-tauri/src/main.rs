// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// Application code for a splashscreen system that waits on a Rust initialization script
#[cfg(not(feature = "ui"))]
mod rust {
  use std::{thread::sleep, time::Duration};
  use tauri::Manager;

  // this command is here just so the example doesn't throw an error
  #[tauri::command]
  fn close_splashscreen() {}

  pub fn main() {
    tauri::Builder::default()
      .setup(|app| {
        let splashscreen_window = app.get_window("splashscreen").unwrap();
        let main_window = app.get_window("main").unwrap();
        // we perform the initialization code on a new task so the app doesn't freeze
        tauri::async_runtime::spawn(async move {
          println!("Initializing...");
          sleep(Duration::from_secs(2));
          println!("Done initializing.");

          // After it's done, close the splashscreen and display the main window
          splashscreen_window.close().unwrap();
          main_window.show().unwrap();
        });
        Ok(())
      })
      .invoke_handler(tauri::generate_handler![close_splashscreen])
      .run(tauri::generate_context!())
      .expect("failed to run app");
  }
}

// Application code for a splashscreen system that waits for the UI
#[cfg(feature = "ui")]
mod ui {
  use std::sync::{Arc, Mutex};
  use tauri::{Manager, Params, State, Window};

  // wrappers around each Window
  // we use a dedicated type because Tauri can only manage a single instance of a given type
  struct SplashscreenWindow<P: Params>(Arc<Mutex<Window<P>>>);
  struct MainWindow<P: Params>(Arc<Mutex<Window<P>>>);

  #[tauri::command]
  fn close_splashscreen<P: Params>(
    splashscreen: State<'_, SplashscreenWindow<P>>,
    main: State<'_, MainWindow<P>>,
  ) {
    // Close splashscreen
    splashscreen.0.lock().unwrap().close().unwrap();
    // Show main window
    main.0.lock().unwrap().show().unwrap();
  }

  pub fn main() {
    tauri::Builder::default()
      .setup(|app| {
        // set the splashscreen and main windows to be globally available with the tauri state API
        app.manage(SplashscreenWindow(Arc::new(Mutex::new(
          app.get_window("splashscreen").unwrap(),
        ))));
        app.manage(MainWindow(Arc::new(Mutex::new(
          app.get_window("main").unwrap(),
        ))));
        Ok(())
      })
      .invoke_handler(tauri::generate_handler![close_splashscreen])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

fn main() {
  #[cfg(feature = "ui")]
  ui::main();
  #[cfg(not(feature = "ui"))]
  rust::main();
}
