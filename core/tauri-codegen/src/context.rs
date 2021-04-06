use crate::assets::{DiskAssets, EmbeddedAssets, Error};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::path::PathBuf;
use tauri_api::config::Config;

/// Necessary data needed by [`codegen_context`] to generate code for a Tauri application context.
pub struct ContextData {
  pub config: Config,
  pub config_parent: PathBuf,
  pub context_path: TokenStream,
}

/// Build an `AsTauriContext` implementation for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, Error> {
  let ContextData {
    mut config,
    config_parent,
    context_path,
  } = data;
  let dist_dir = config_parent.join(&config.build.dist_dir);
  config.build.dist_dir = dist_dir.to_string_lossy().to_string();

  // generate an appropriate asset container based on if this build is a debug build
  let assets = if cfg!(debug_assertions) {
    DiskAssets::new(&dist_dir)?.to_token_stream()
  } else {
    EmbeddedAssets::new(&dist_dir)?.to_token_stream()
  };

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
    package_info: ::tauri::api::PackageInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION")
    }
  }))
}
