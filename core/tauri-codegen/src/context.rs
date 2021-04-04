use crate::embedded_assets::{EmbeddedAssets, EmbeddedAssetsError};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;
use tauri_api::config::Config;

/// Necessary data needed by [`codegen_context`] to generate code for a Tauri application context.
pub struct ContextData {
  pub config: Config,
  pub config_parent: PathBuf,
  pub context_path: TokenStream,
}

/// Build an `AsTauriContext` implementation for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, EmbeddedAssetsError> {
  let ContextData {
    mut config,
    config_parent,
    context_path,
  } = data;
  let dist_dir = config_parent.join(&config.build.dist_dir);
  config.build.dist_dir = dist_dir.to_string_lossy().to_string();

  // generate the assets inside the dist dir into a perfect hash function
  let assets = EmbeddedAssets::new(&dist_dir)?;

  // handle default window icons for Windows targets
  let default_window_icon = if cfg!(windows) {
    let icon_path = config_parent.join("icons/icon.ico").display().to_string();
    quote!(Some(include_bytes!(#icon_path).to_vec()))
  } else {
    quote!(None)
  };

  // double braces are purposeful to force the code into a block expression
  Ok(quote!(#context_path {
    config: #config,
    assets: #assets,
    default_window_icon: #default_window_icon,
  }))
}
