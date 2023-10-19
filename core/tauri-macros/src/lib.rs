// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/tauri/raw/dev/.github/splash.png)](https://tauri.app)
//!
//! Create macros for `tauri::Context`, invoke handler and commands leveraging the `tauri-codegen` crate.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use crate::context::ContextItems;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod command;
mod mobile;
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

#[proc_macro_attribute]
pub fn mobile_entry_point(attributes: TokenStream, item: TokenStream) -> TokenStream {
  mobile::entry_point(attributes, item)
}

/// Accepts a list of commands functions. Creates a handler that allows commands to be called from JS with invoke().
///
/// # Examples
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
