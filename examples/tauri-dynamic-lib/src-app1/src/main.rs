// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This is an example of an application that loads and runs a dll
// Typically this could be c++, we use rust here just for convenience
// See https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use libloading::{library_filename, Library, Symbol};
type LibFunctionType1 = fn();

fn main() {
  let library_path = library_filename("../src-tauri/target/debug/tauri_app");
  println!("Loading run_tauri() from {:?}", library_path);

  unsafe {
    let lib = Library::new(library_path).unwrap();
    let run_tauri: Symbol<LibFunctionType1> = lib.get(b"run_tauri").unwrap();
    println!("Launching webview");
    run_tauri()
  }
}
