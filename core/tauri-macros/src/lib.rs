// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::context::ContextItems;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod command;
mod command_module;
mod runtime;

#[macro_use]
mod context;

/// Mark a function as a command handler. It creates a wrapper function with the necessary glue code.
///
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro_attribute]
pub fn command(attributes: TokenStream, item: TokenStream) -> TokenStream {
  command::wrapper(attributes, item)
}

/// Accepts a list of commands functions. Creates a handler that allows commands to be called from JS with invoke().
///
/// # Example
/// ```rust,ignore
/// use tauri_macros::{command, generate_handler};
/// #[command]
/// fn command_one() {
///   println!("command one called");
/// }
/// #[command]
/// fn command_two() {
///   println!("command two called");
/// }
/// fn main() {
///   let _handler = generate_handler![command_one, command_two];
/// }
/// ```
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  parse_macro_input!(item as command::Handler).into()
}

/// Reads a Tauri config file and generates a `::tauri::Context` based on the content.
///
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro]
pub fn generate_context(items: TokenStream) -> TokenStream {
  // this macro is exported from the context module
  let path = parse_macro_input!(items as ContextItems);
  context::generate_context(path).into()
}

/// Adds the default type for the last parameter (assumed to be runtime) for a specific feature.
///
/// e.g. To default the runtime generic to type `crate::Wry` when the `wry` feature is enabled, the
/// syntax would look like `#[default_runtime(crate::Wry, wry)`. This is **always** set for the last
/// generic, so make sure the last generic is the runtime when using this macro.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn default_runtime(attributes: TokenStream, input: TokenStream) -> TokenStream {
  let attributes = parse_macro_input!(attributes as runtime::Attributes);
  let input = parse_macro_input!(input as DeriveInput);
  runtime::default_runtime(attributes, input).into()
}

/// Adds a `run` method to an enum (one of the tauri endpoint modules).
/// The `run` method takes a `tauri::endpoints::InvokeContext`
/// and returns a `tauri::Result<tauri::endpoints::InvokeResponse>`.
/// It matches on each enum variant and call a method with name equal to the variant name, lowercased and snake_cased,
/// passing the the context and the variant's fields as arguments.
/// That function must also return the same `Result<InvokeResponse>`.
#[doc(hidden)]
#[proc_macro_derive(CommandModule, attributes(cmd))]
pub fn derive_command_module(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  command_module::generate_run_fn(input)
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn module_command_handler(attributes: TokenStream, input: TokenStream) -> TokenStream {
  let attributes = parse_macro_input!(attributes as command_module::HandlerAttributes);
  let input = parse_macro_input!(input as ItemFn);
  command_module::command_handler(attributes, input).into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn module_command_test(attributes: TokenStream, input: TokenStream) -> TokenStream {
  let attributes = parse_macro_input!(attributes as command_module::HandlerTestAttributes);
  let input = parse_macro_input!(input as ItemFn);
  command_module::command_test(attributes, input).into()
}
