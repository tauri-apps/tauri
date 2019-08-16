extern crate includedir_codegen;

use includedir_codegen::Compression;
use std::env;

static CARGOENV: &str = "cargo:rustc-env=";

fn main() {
  let dist_path = format!("{}/../../../../{}", env::var("OUT_DIR").unwrap(), "compiled-web");
  println!("{}TAURI_DIST_DIR={}", CARGOENV, dist_path);
  includedir_codegen::start("ASSETS")
    .dir(dist_path, Compression::None)
    .build("data.rs")
    .unwrap();
}
