use crate::error::Error;
use crate::include_dir::IncludeDir;
use crate::{Env, DEFAULT_CONFIG_FILE};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::path::Path;
use syn::{DeriveInput, Lit::Str, Meta::NameValue, MetaNameValue};
use tauri_api::assets::Compression;
use tauri_api::config::Config;

pub(crate) fn from_tauri_config(input: DeriveInput) -> Result<TokenStream, Error> {
  let name = input.ident;
  let env = Env::new()?;

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

  let full_config_path = Path::new(&env.manifest).join(config_file_path);
  let config = Config::read(&full_config_path).unwrap();
  let config_dir = full_config_path.parent().unwrap();
  let dist_dir = config_dir.join(config.build.dist);

  // generate the assets into a perfect hash function
  let assets = generate_assets(&dist_dir)?; //.unwrap();

  // should be possible to do the index.tauri.hmtl manipulations during this macro too in the future
  let tauri_index_html = dist_dir.join("index.tauri.html");

  // format paths into a string to use them in quote!
  let tauri_config_path = full_config_path.display().to_string();
  let tauri_index_html = tauri_index_html.display().to_string();

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
            use ::tauri::api::assets::{Assets, phf, phf::phf_map};
            static ASSETS: Assets = #assets;
            &ASSETS
          }

          /// Make the index.tauri.html a dependency for the compiler
          fn raw_index() -> &'static str {
            include_str!(#tauri_index_html)
          }
      }
  })
}

/// Generates a perfect hash function from `phf` of the assets in dist directory
///
/// The `TokenStream` produced by this function expects to have `phf` and
/// `phf_map` paths available. Make sure to `use` these so the macro has access to them.
fn generate_assets(dist: &Path) -> Result<TokenStream, Error> {
  let mut inline_assets = HashSet::new();
  if let Ok(assets) = std::env::var("TAURI_INLINED_ASSETS") {
    assets
      .split('|')
      .filter(|&s| !s.trim().is_empty())
      .for_each(|s| {
        inline_assets.insert(s.into());
      })
  }

  // the index.html is parsed so we always ignore it
  inline_assets.insert(
    dist
      .join("index.html")
      .into_os_string()
      .into_string()
      .expect("failed to convert dist path to string"),
  );

  // TODO: disabled because I dont know how this case should be handled yet (cfg)
  /*
  if cfg!(feature = "no-server") {
    // on no-server we include_str() the index.tauri.html on the runner
    inlined_assets.push(
      dist
        .join("index.tauri.html")
        .into_os_string()
        .into_string()
        .expect("failed to convert dist path to string"),
    );
  }
  */

  IncludeDir::new(&dist)
    .dir(&dist, Compression::None)?
    .set_filter(inline_assets)
    .build()
}
