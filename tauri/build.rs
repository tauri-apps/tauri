#[cfg(not(feature = "dev-server"))]
pub fn main() {
  match std::env::var_os("TAURI_DIST_DIR") {
    Some(dist_path) => {
      println!(
        "cargo:rerun-if-changed={}",
        dist_path.into_string().unwrap()
      );

      let inlined_assets = match std::env::var_os("TAURI_INLINED_ASSETS") {
        Some(assets) => assets
          .into_string()
          .unwrap()
          .split('|')
          .map(|s| s.to_string())
          .collect(),
        None => Vec::new(),
      };
      // include assets
      tauri_includedir_codegen::start("ASSETS")
        .dir(
          dist_path.into_string().unwrap(),
          tauri_includedir_codegen::Compression::None,
        )
        .build("data.rs", inlined_assets)
        .expect("failed to build data.rs")
    }
    None => {
      println!("Build error: Couldn't find ENV: TAURI_DIST_DIR");
      ()
    }
  }
}

#[cfg(feature = "dev-server")]
pub fn main() {}
