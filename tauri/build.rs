#[cfg(any(feature = "embedded-server", feature = "no-server"))]
use std::{
  env,
  error::Error,
  fs::{read_to_string, File},
  io::{BufWriter, Write},
  path::Path,
};

#[cfg(any(feature = "embedded-server", feature = "no-server"))]
pub fn main() -> Result<(), Box<dyn Error>> {
  shared();

  let out_dir = env::var("OUT_DIR")?;

  let dest_index_html_path = Path::new(&out_dir).join("index.tauri.html");
  let mut index_html_file = BufWriter::new(File::create(&dest_index_html_path)?);

  match env::var_os("TAURI_DIST_DIR") {
    Some(dist_path) => {
      let dist_path_string = dist_path.into_string().unwrap();

      println!("cargo:rerun-if-changed={}", dist_path_string);

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
          dist_path_string.clone(),
          tauri_includedir_codegen::Compression::None,
        )
        .build("data.rs", inlined_assets)
        .expect("failed to build data.rs");

      let original_index_html_path = Path::new(&dist_path_string).join("index.tauri.html");
      let original_index_html = read_to_string(original_index_html_path)?;

      write!(index_html_file, "{}", original_index_html)?;
    }
    None => {
      // dummy assets
      tauri_includedir_codegen::start("ASSETS")
        .dir("".to_string(), tauri_includedir_codegen::Compression::None)
        .build("data.rs", vec![])
        .expect("failed to build data.rs");
      write!(
        index_html_file,
        "<html><body>Build error: Couldn't find ENV: TAURI_DIST_DIR</body></html>"
      )?;
      println!("Build error: Couldn't find ENV: TAURI_DIST_DIR");
    }
  }
  Ok(())
}

#[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
pub fn main() {
  shared();
}

fn shared() {
  if let Some(tauri_dir) = std::env::var_os("TAURI_DIR") {
    let mut tauri_path = std::path::PathBuf::from(tauri_dir);
    tauri_path.push("tauri.conf.json");
    println!("cargo:rerun-if-changed={:?}", tauri_path);
  }
}
