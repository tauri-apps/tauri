extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod error;
mod expand;
mod include_dir;

const DEFAULT_CONFIG_FILE: &str = "tauri.conf.json";

#[proc_macro_derive(FromTauriConfig, attributes(tauri_config_path))]
pub fn from_tauri_config(ast: TokenStream) -> TokenStream {
  let input = parse_macro_input!(ast as DeriveInput);
  let name = input.ident.clone();

  expand::from_tauri_config(input)
    .unwrap_or_else(|e| e.into_compile_error(&name))
    .into()
}
