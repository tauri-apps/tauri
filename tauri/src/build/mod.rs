use std::{
  collections::HashSet,
  fs::File,
  io::{BufReader, BufWriter, Write},
  path::{Path, PathBuf},
};

use anyhow::{Context, Error};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
  api::{assets::AssetCompression, config::Config},
  build::include_dir::IncludeDir,
};

mod include_dir;

// same level as `build.rs` where we are expecting `do_build` to be called from.
const DEFAULT_CONFIG_PATH: &str = "tauri.conf.json";

/// build an `AsTauriContext` snippet to include in application code
pub fn do_build(config_path: Option<PathBuf>) -> Result<(), Error> {
  let config_path = std::env::current_dir()
    .with_context(|| "Unable to access the current directory")
    .map(|p| p.join(config_path.unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))))?;
  let config = get_config(&config_path)
    .with_context(|| format!("Failed to read config file from {}", config_path.display()))?;

  let config_dir = config_path.parent().with_context(|| {
    format!(
      "Unable to access parent dir of config file {}",
      config_path.display()
    )
  })?;
  let dist_dir = config_dir.join(&config.build.dist_dir);

  // generate the assets into a perfect hash function
  let assets = generate_asset_map(&dist_dir)?;

  let default_window_icon = if cfg!(windows) {
    let icon_path = config_dir.join("./icons/icon.ico").display().to_string();
    quote! {
      Some(include_bytes!(#icon_path))
    }
  } else {
    quote! { None }
  };

  let tauri_script_path = dist_dir.join("__tauri.js");

  // format paths into a string to use them in quote!
  let tauri_script_path = tauri_script_path.display().to_string();

  #[allow(non_snake_case)]
  let AutoStruct = quote::format_ident!("AutoTauriContext");

  // double braces are purposeful to force the code into a block expression
  let scoped_expression = quote! {{
    use ::tauri::api::config::Config;
    use ::tauri::api::assets::{Assets, AssetCompression, phf, phf::phf_map};
    use ::tauri::api::private::{OnceCell, AsTauriContext};

    struct #AutoStruct;

    static INSTANCE: OnceCell<Config> = OnceCell::new();

    impl AsTauriContext for #AutoStruct {
        /// Return a static reference to the config we parsed at build time
        fn config() -> &'static Config {
            INSTANCE.get_or_init(|| #config)
        }

        /// Inject assets we generated during build time
        fn assets() -> &'static Assets {
          static ASSETS: Assets = Assets::new(#assets);
          &ASSETS
        }

        /// Make the __tauri.js a dependency for the compiler
        fn raw_tauri_script() -> &'static str {
          include_str!(#tauri_script_path)
        }

        /// Default window icon to set automatically if exists
        fn default_window_icon() -> Option<&'static [u8]> {
          #default_window_icon
        }
      }

      #AutoStruct{}
  }};

  let out = std::env::var("OUT_DIR")
    .with_context(|| "unable to find OUT_DIR from tauri-build")
    .map(PathBuf::from)?;
  let out = out.join("tauri_config.rs");

  let file = File::create(&out).with_context(|| {
    format!(
      "Unable to create out file during tauri-build {}",
      out.display()
    )
  })?;
  let mut file = BufWriter::new(file);
  writeln!(&mut file, "{}", scoped_expression).with_context(|| {
    format!(
      "Unable to write tokenstream to out file during tauri-build {}",
      out.display()
    )
  })?;

  Ok(())
}

fn get_config(path: &Path) -> Result<Config, Error> {
  match std::env::var("TAURI_CONFIG") {
    Ok(custom_config) => serde_json::from_str(&custom_config).with_context(|| {
      format!(
        "Unable to parse inline env var TAURI_CONFIG\nTAURI_CONFIG: {}",
        &custom_config
      )
    }),
    Err(_) => {
      let file = File::open(&path)
        .with_context(|| format!("Unable to open config file at path {}", path.display()))?;
      let reader = BufReader::new(file);
      serde_json::from_reader(reader).with_context(|| {
        format!(
          "Unable to parse json config file at path: {}",
          path.display()
        )
      })
    }
  }
}

/// Generates a perfect hash function from `phf` of the assets in dist directory
///
/// The `TokenStream` produced by this function expects to have `phf` and
/// `phf_map` paths available. Make sure to `use` these so the macro has access to them.
/// It also expects `AssetCompression` to be in path.
fn generate_asset_map(dist: &Path) -> Result<TokenStream, Error> {
  let mut inline_assets = HashSet::new();
  if let Ok(assets) = std::env::var("TAURI_INLINED_ASSETS") {
    assets
      .split('|')
      .filter(|&s| !s.trim().is_empty())
      .map(PathBuf::from)
      .for_each(|path| {
        inline_assets.insert(path);
      })
  }

  IncludeDir::new(&dist)
    .dir(&dist, AssetCompression::Gzip)?
    .set_filter(inline_assets)?
    .build()
}
