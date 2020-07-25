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
  match expand::from_tauri_config(input) {
    Ok(ast) => ast.into(),
    Err(err) => err.into(),
  }
}
