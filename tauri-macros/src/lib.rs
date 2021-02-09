extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod error;
mod expand;
mod include_dir;

const DEFAULT_CONFIG_FILE: &str = "tauri.conf.json";

#[proc_macro_derive(FromTauriContext, attributes(config_path))]
pub fn load_context(ast: TokenStream) -> TokenStream {
  let input = parse_macro_input!(ast as DeriveInput);
  let name = input.ident.clone();

  expand::load_context(input)
    .unwrap_or_else(|e| e.into_compile_error(&name))
    .into()
}
