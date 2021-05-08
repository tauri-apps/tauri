// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

extern crate proc_macro;
use crate::context::ContextItems;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

mod command;

#[macro_use]
mod context;

#[proc_macro_attribute]
pub fn command(attributes: TokenStream, item: TokenStream) -> TokenStream {
  let attributes = parse_macro_input!(attributes as AttributeArgs);
  let function = parse_macro_input!(item as ItemFn);
  match command::Wrapper::new(function, attributes) {
    Ok(wrapper) => wrapper.into(),
    Err(error) => error.into_compile_error().into(),
  }
}

#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  parse_macro_input!(item as command::Handler).into()
}

/// Reads a Tauri config file and generates a `::tauri::Context` based on the content.
#[proc_macro]
pub fn generate_context(items: TokenStream) -> TokenStream {
  // this macro is exported from the context module
  let path = parse_macro_input!(items as ContextItems);
  context::generate_context(path).into()
}
