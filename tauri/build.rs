#[cfg(not(feature = "dev-server"))]
extern crate tauri_includedir_codegen;

pub fn main() {
  #[cfg(not(feature = "dev-server"))]
  {
    println!("cargo:rerun-if-changed={}", std::env::var("TAURI_DIST_DIR"));
    match std::env::var("TAURI_DIST_DIR") {
      Ok(dist_path) => {
        let inlined_assets = match std::env::var("TAURI_INLINED_ASSETS") {
          Ok(assets) => assets.split("|").map(|s| s.to_string()).collect(),
          Err(_) => Vec::new(),
        };
        // include assets
        tauri_includedir_codegen::start("ASSETS")
          .dir(dist_path, tauri_includedir_codegen::Compression::None)
          .build("data.rs", inlined_assets)
          .expect("failed to build data.rs")
      }
      Err(e) => panic!("Build error: Couldn't find ENV: {}", e),
    }
  }
}
