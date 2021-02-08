use crate::error::Error;
use crate::include_dir::IncludeDir;
use crate::DEFAULT_CONFIG_FILE;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::env::var;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use syn::{DeriveInput, Lit::Str, Meta::NameValue, MetaNameValue};
use tauri_api::assets::AssetCompression;
use tauri_api::config::Config;

pub(crate) fn from_tauri_config(input: DeriveInput) -> Result<TokenStream, Error> {
  let name = input.ident;

  // quick way of parsing #[tauri_config_path = "path_goes_here"]
  let mut config_file_path = DEFAULT_CONFIG_FILE.into();
  let tauri_config_path_attr = input
    .attrs
    .iter()
    .find(|attr| attr.path.is_ident("tauri_config_path"));
  if let Some(attr) = tauri_config_path_attr {
    if let Ok(meta) = attr.parse_meta() {
      if let NameValue(MetaNameValue { lit: Str(path), .. }) = meta {
        config_file_path = path.value()
      }
    }
  }

  // grab the manifest of the application the macro is in
  let manifest = var("CARGO_MANIFEST_DIR")
    .map(PathBuf::from)
    .map_err(|_| Error::EnvCargoManifestDir)?;

  let full_config_path = Path::new(&manifest).join(config_file_path);
  let config = get_config(&full_config_path)?;
  let config_dir = full_config_path.parent().ok_or(Error::ConfigDir)?;
  let dist_dir = config_dir.join(config.build.dist_dir);

  // generate the assets into a perfect hash function
  let assets = generate_asset_map(&dist_dir)?;

  // should be possible to do the index.tauri.hmtl manipulations during this macro too in the future
  let tauri_index_html_path = dist_dir.join("index.tauri.html");
  let tauri_script_path = dist_dir.join("__tauri.js");

  // format paths into a string to use them in quote!
  let tauri_config_path = full_config_path.display().to_string();
  let tauri_index_html_path = tauri_index_html_path.display().to_string();
  let tauri_script_path = tauri_script_path.display().to_string();

  Ok(quote! {
      impl ::tauri::api::private::AsTauriConfig for #name {
          fn config_path() -> &'static std::path::Path {
              std::path::Path::new(#tauri_config_path)
          }

          /// Make the file a dependency for the compiler
          fn raw_config() -> &'static str {
            include_str!(#tauri_config_path)
          }

          fn assets() -> &'static ::tauri::api::assets::Assets {
            use ::tauri::api::assets::{Assets, AssetCompression, phf, phf::phf_map};
            static ASSETS: Assets = Assets::new(#assets);
            &ASSETS
          }

          /// Make the index.tauri.html a dependency for the compiler
          fn raw_index() -> &'static str {
            include_str!(#tauri_index_html_path)
          }

          /// Make the __tauri.js a dependency for the compiler
          fn raw_tauri_script() -> &'static str {
            include_str!(#tauri_script_path)
          }
      }
  })
}

fn get_config(path: &Path) -> Result<Config, Error> {
  match var("TAURI_CONFIG") {
    Ok(custom_config) => {
      serde_json::from_str(&custom_config).map_err(|e| Error::Serde("TAURI_CONFIG".into(), e))
    }
    Err(_) => {
      let file = File::open(&path).map_err(|e| Error::Io(path.into(), e))?;
      let reader = BufReader::new(file);
      serde_json::from_reader(reader).map_err(|e| Error::Serde(path.into(), e))
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

  // the index.html is parsed so we always ignore it
  inline_assets.insert("/index.html".into());

  IncludeDir::new(&dist)
    .dir(&dist, AssetCompression::Gzip)?
    .set_filter(inline_assets)?
    .build()
}
