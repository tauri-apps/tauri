// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::embedded_assets::{AssetOptions, EmbeddedAssets, EmbeddedAssetsError};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;
use tauri_utils::config::Config;

/// Necessary data needed by [`context_codegen`] to generate code for a Tauri application context.
pub struct ContextData {
  pub dev: bool,
  pub config: Config,
  pub config_parent: PathBuf,
  pub root: TokenStream,
}

/// Build a `tauri::Context` for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, EmbeddedAssetsError> {
  let ContextData {
    dev,
    config,
    config_parent,
    root,
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

  // generate the assets inside the dist dir into a perfect hash function
  let assets = if let Some(assets_path) = assets_path {
    let mut options = AssetOptions::new();
    if let Some(csp) = &config.tauri.security.csp {
      options = options.csp(csp.clone());
    }
    EmbeddedAssets::new(&assets_path, options)?
  } else {
    Default::default()
  };

  // handle default window icons for Windows targets
  let default_window_icon = if cfg!(windows) {
    let icon_path = config
      .tauri
      .bundle
      .icon
      .iter()
      .find(|i| i.ends_with(".ico"))
      .cloned()
      .unwrap_or_else(|| "icons/icon.ico".to_string());
    let icon_path = config_parent.join(icon_path).display().to_string();
    quote!(Some(include_bytes!(#icon_path).to_vec()))
  } else {
    quote!(None)
  };

  let package_name = if let Some(product_name) = &config.package.product_name {
    quote!(#product_name.to_string())
  } else {
    quote!(env!("CARGO_PKG_NAME").to_string())
  };
  let package_version = if let Some(version) = &config.package.version {
    quote!(#version.to_string())
  } else {
    quote!(env!("CARGO_PKG_VERSION").to_string())
  };
  let package_info = quote!(
    #root::api::PackageInfo {
      name: #package_name,
      version: #package_version,
    }
  );

  #[cfg(target_os = "linux")]
  let system_tray_icon = if let Some(tray) = &config.tauri.system_tray {
    let mut system_tray_icon_path = tray.icon_path.clone();
    system_tray_icon_path.set_extension("png");
    if dev {
      let system_tray_icon_path = config_parent
        .join(system_tray_icon_path)
        .display()
        .to_string();
      quote!(Some(#root::Icon::File(::std::path::PathBuf::from(#system_tray_icon_path))))
    } else {
      let system_tray_icon_file_path = system_tray_icon_path.to_string_lossy().to_string();
      quote!(
        Some(
          #root::Icon::File(
            #root::api::path::resolve_path(
              &#config, &#package_info,
             #system_tray_icon_file_path,
             Some(#root::api::path::BaseDirectory::Resource)
            ).expect("failed to resolve resource dir")
          )
        )
      )
    }
  } else {
    quote!(None)
  };

  #[cfg(not(target_os = "linux"))]
  let system_tray_icon = if let Some(tray) = &config.tauri.system_tray {
    let mut system_tray_icon_path = tray.icon_path.clone();
    system_tray_icon_path.set_extension(if cfg!(windows) { "ico" } else { "png" });
    let system_tray_icon_path = config_parent
      .join(system_tray_icon_path)
      .display()
      .to_string();
    quote!(Some(#root::Icon::Raw(include_bytes!(#system_tray_icon_path).to_vec())))
  } else {
    quote!(None)
  };

  // double braces are purposeful to force the code into a block expression
  Ok(quote!(#root::Context::new(
    #config,
    ::std::sync::Arc::new(#assets),
    #default_window_icon,
    #system_tray_icon,
    #package_info,
  )))
}
