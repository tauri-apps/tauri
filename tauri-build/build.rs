#[cfg(not(feature = "dev-server"))]
extern crate tauri_includedir_codegen;

#[cfg(not(feature = "dev-server"))]
#[macro_use]
extern crate serde_derive;
#[cfg(not(feature = "dev-server"))]
extern crate serde_json;

#[cfg(not(feature = "dev-server"))]
#[path = "src/config.rs"]
mod config;

pub fn main() {
  #[cfg(not(feature = "dev-server"))]
  {
    match std::env::var("TAURI_DIST_DIR") {
      Ok(dist_path) => {
        let config = config::get_tauri_dir();
        // include assets
        tauri_includedir_codegen::start("ASSETS")
          .dir(dist_path, tauri_includedir_codegen::Compression::None)
          .build("data.rs", config.inlined_assets)
          .expect("failed to build data.rs")
      }
      Err(e) => panic!("Build error: Couldn't find ENV: {}", e),
    }
  }
}