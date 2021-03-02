extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

mod command;

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
