extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

mod command;

#[macro_use]
mod context;

#[proc_macro_attribute]
pub fn command(attrs: TokenStream, item: TokenStream) -> TokenStream {
  let function = parse_macro_input!(item as ItemFn);
  let attrs = parse_macro_input!(attrs as AttributeArgs);
  let gen = command::generate_command(attrs, function);
  gen.into()
}

#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  let gen = command::generate_handler(item);
  gen.into()
}

/// Reads a Tauri config file and generates an [`AsTauriContext`] based on the content.
///
/// The default config file path is a `tauri.conf.json` file inside the Cargo manifest directory of
/// the crate being built.
///
/// # Custom Config Path
///
/// You may pass a string literal to this macro to specify a custom path for the Tauri config file.
/// If the path is relative, it will be search for relative to the Cargo manifest of the compiling
/// crate.
///
/// todo: link the [`AsTauriContext`] docs
#[proc_macro]
pub fn generate_context(item: TokenStream) -> TokenStream {
  // this macro is exported from the context module
  let path = parse_config_path!(item);
  context::generate_context(path).into()
}
