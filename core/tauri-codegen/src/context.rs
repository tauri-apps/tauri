use crate::embedded_assets::{EmbeddedAssets, EmbeddedAssetsError};
use proc_macro2::TokenStream;
use quote::quote;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use tauri_api::config::Config;

/// Necessary data needed by [`codegen_context`] to generate code for a Tauri application context.
pub struct ContextData {
  pub dev: bool,
  pub config: Config,
  pub config_parent: PathBuf,
  pub context_path: TokenStream,
}

/// Build an `AsTauriContext` implementation for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, EmbeddedAssetsError> {
  let ContextData {
    dev,
    config,
    config_parent,
    context_path,
  } = data;
  let assets_path = if dev {
    // if dev_path is a dev server, we don't have any assets to embed
    if config.build.dev_path.starts_with("http") {
      None
    } else {
      Some(config_parent.join(&config.build.dev_path))
    }
  } else {
    Some(config_parent.join(&config.build.dist_dir))
  };

  let config_copy_path = crate::out_dir()?.join("tauri.config.json");
  let config_copy = File::create(&config_copy_path)
    .map(BufWriter::new)
    .map_err(|error| EmbeddedAssetsError::AssetWrite {
      path: config_copy_path.clone(),
      error,
    })?;
  serde_json::to_writer(config_copy, &config).expect("unable to serialize known good config");

  // generate the assets inside the dist dir into a perfect hash function
  let assets = if let Some(assets_path) = assets_path {
    EmbeddedAssets::new(&assets_path)?
  } else {
    Default::default()
  };

  // handle default window icons for Windows targets
  let default_window_icon = if cfg!(windows) {
    let icon_path = config_parent.join("icons/icon.ico").display().to_string();
    quote!(Some(include_bytes!(#icon_path).to_vec()))
  } else {
    quote!(None)
  };

  let config = config_copy_path.display().to_string();

  // double braces are purposeful to force the code into a block expression
  Ok(quote!(#context_path {
    config: ::tauri::api::private::serde_json::from_slice(include_bytes!(#config)).expect("bad config format"),
    assets: #assets,
    default_window_icon: #default_window_icon,
    package_info: ::tauri::api::PackageInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION")
    }
  }))
}
