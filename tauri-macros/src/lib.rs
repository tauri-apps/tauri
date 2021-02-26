extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse::Parser, parse_macro_input, Attribute, DeriveInput};

mod command;
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

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
  let function: syn::ItemFn = syn::parse2(item.clone().into()).unwrap();
  let attrs = Attribute::parse_outer.parse2(attr.into()).unwrap();
  let gen = command::generate_command(attrs, function);
  gen.into()
}

#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  let gen = command::generate_handler(item);
  gen.into()
}
