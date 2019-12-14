use includedir_codegen::Compression;
use std::env;

fn main() {
  let var = if cfg!(feature = "development") {
    Ok(String::from("../examples/gatsby/themed-site"))
  } else {
    env::var("TAURI_DIST_DIR")
  };

  match var {
    Ok(dist_path) => includedir_codegen::start("ASSETS")
      .dir(dist_path, Compression::None)
      .build("data.rs")
      .unwrap(),
    Err(e) => panic!("Build error: Cound't find ENV: {}", e),
  }
}
