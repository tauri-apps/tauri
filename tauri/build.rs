use includedir_codegen::Compression;
use std::env;

fn main() {
  match env::var('TAURI_DIST_DIR') {
    Ok(dist_path) => includedir_codegen::start("ASSETS")
      .dir(dist_path, Compression::None)
      .build("data.rs")
      .unwrap(),
    Err(_e) => println!("Bail: Couldn't find ENV: {}", _e),
  }
}
