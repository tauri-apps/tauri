// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg_attr(feature = "cef", tauri::cef_entry_point)]
fn main() {
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::<tauri::Cef>::default();
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::<tauri::Wry>::new();

  let app = builder
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
