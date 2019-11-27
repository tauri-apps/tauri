use includedir_codegen::Compression;
use std::env;

fn main() {
  let dist_path = env::var("TAURI_DIST_DIR").unwrap();
  includedir_codegen::start("ASSETS")
    .dir(dist_path, Compression::None)
    .build("data.rs")
    .unwrap();
}
