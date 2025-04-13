// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let app = tauri::Builder::default()
    .build(tauri::generate_context!(
      "../../examples/run-return/tauri.conf.json"
    ))
    .expect("error while building tauri application");

  let exit_code = app.run_return(|_app, _event| {
    //println!("{:?}", _event);
  });

  println!("I run after exit");

  std::process::exit(exit_code);
}
