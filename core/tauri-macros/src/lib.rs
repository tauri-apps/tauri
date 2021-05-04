// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

extern crate proc_macro;
use crate::context::ContextItems;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod command;

#[macro_use]
mod context;

#[proc_macro_attribute]
pub fn command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
  let function = parse_macro_input!(item as ItemFn);
  let gen = command::generate_command(function);
  gen.into()
}

#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  let gen = command::generate_handler(item);
  gen.into()
}

/// Reads a Tauri config file and generates a `::tauri::Context` based on the content.
#[proc_macro]
pub fn generate_context(items: TokenStream) -> TokenStream {
  // this macro is exported from the context module
  let path = parse_macro_input!(items as ContextItems);
  context::generate_context(path).into()
}
