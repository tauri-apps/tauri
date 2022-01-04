// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This is an example of an application that loads and runs a dll
// Typically this could be c++, we use rust here just for convenience
// See https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use libloading::{Library, Symbol};
type LibFunctionType1 = fn();

fn main() {
  let library_path = "../src-dll1/target/debug/dll1.dll";
  println!("Loading lib_test1() from {}", library_path);

  unsafe {
    let lib = Library::new(library_path).unwrap();
    let _func: Symbol<LibFunctionType1> = lib.get(b"lib_test1").unwrap();
    println!("Launching webview");
    _func()
  }
}
