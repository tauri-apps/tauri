use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::PathBuf;
use tauri_codegen::{context_codegen, get_config, ContextData};

/// Parse the passed [`proc_macro::TokenStream`] and make sure the config file exists.
///
/// The [`proc_macro::TokenStream`] is expected to be empty for the default config path, or passed
/// a [`syn::LitStr`] a custom config path. The custom path can be either relative or absolute, with
/// relative paths being searched from the current working directory of the compiling crate.
///
/// This is a macro so that we can use [`syn::parse_macro_input!`] and easily return a
/// [`std::compile_error!`] when the path input is bad.
#[macro_use]
macro_rules! parse_config_path {
  ($path:ident) => {
    {
      use ::std::{env::{var, VarError}, path::PathBuf};
      use ::syn::{parse_macro_input, LitStr};
      use ::quote::quote;

      let path = if $path.is_empty() {
        var("CARGO_MANIFEST_DIR").map(|m| PathBuf::from(m).join("tauri.conf.json"))
      } else {
        let raw = parse_macro_input!($path as LitStr);
        let path = PathBuf::from(raw.value());
        if path.is_relative() {
          var("CARGO_MANIFEST_DIR").map(|m| PathBuf::from(m).join(path))
        } else {
          Ok(path)
        }
      };

      let path = path
        .map_err(|error| match error {
          VarError::NotPresent => "no CARGO_MANIFEST_DIR env var, this should be set by cargo".into(),
          VarError::NotUnicode(_) => "CARGO_MANIFEST_DIR env var contained invalid utf8".into()
        })
        .and_then(|path| {
          if path.exists() {
            Ok(path)
          } else {
            Err(format!(
              "no file at path {} exists, expected tauri config file",
              path.display()
            ))
          }
        });

      match path {
        Ok(path) => path,
        Err(error_string) => return quote!(compile_error!(#error_string)).into(),
      }
    }
  }
}

pub(crate) fn generate_context(path: PathBuf) -> TokenStream {
  let context = get_config(&path)
    .map_err(|e| e.to_string())
    .map(|(config, config_parent)| ContextData {
      config,
      config_parent,
      struct_ident: format_ident!("AutoTauriContext"),
    })
    .and_then(|data| context_codegen(data).map_err(|e| e.to_string()));

  match context {
    Ok(code) => code,
    Err(error) => quote!(compile_error!(#error)),
  }
}
