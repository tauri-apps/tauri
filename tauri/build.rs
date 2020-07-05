use cfg_aliases::cfg_aliases;

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

      let mut inlined_assets = match std::env::var_os("TAURI_INLINED_ASSETS") {
        Some(assets) => assets
          .into_string()
          .unwrap()
          .split('|')
          .map(|s| s.to_string())
          .filter(|s| s != "")
          .collect(),
        None => Vec::new(),
      };

      // the index.html is parsed so we always ignore it
      inlined_assets.push("index.html".to_string());
      if cfg!(feature = "no-server") {
        // on no-server we include_str() the index.tauri.html on the runner
        inlined_assets.push("index.tauri.html".to_string());
      }

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

  cfg_aliases! {
    embedded_server: { feature = "embedded-server" },
    no_server: { feature = "no-server" },
    assets: { any(feature = "embedded-server", feature = "no-server") },
    dev: { not(any(feature = "embedded-server", feature = "no-server")) },

    all_api: { feature = "all-api" },

    // fs
    read_text_file: { any(all_api, feature = "read-text-file") },
    read_binary_file: { any(all_api, feature = "read-binary-file") },
    write_file: { any(all_api, feature = "write-file") },
    write_binary_file: { any(all_api, feature = "write-binary-file") },
    read_dir: { any(all_api, feature = "read-dir") },
    copy_file: { any(all_api, feature = "copy-file") },
    create_dir: { any(all_api, feature = "create_dir") },
    remove_dir: { any(all_api, feature = "remove-dir") },
    remove_file: { any(all_api, feature = "remove-file") },
    rename_file: { any(all_api, feature = "rename-file") },

    // window
    set_title: { any(all_api, feature = "set-title") },
    open: { any(all_api, feature = "open") },

    // process
    execute: { any(all_api, feature = "execute") },

    // event
    event: { any(all_api, feature = "event") },

    // dialog
    open_dialog: { any(all_api, feature = "open-dialog") },
    save_dialog: { any(all_api, feature = "save-dialog") },

    // http
    http_request: { any(all_api, feature = "http-request") },

    // cli
    cli: { feature = "cli" },

    // notification
    notification: { any(all_api, feature = "notification") },
  }
}
