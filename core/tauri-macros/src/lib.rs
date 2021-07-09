// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

extern crate proc_macro;
use crate::context::ContextItems;
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericParam, TypeParam};

mod command;

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
/// use tauri::command;
/// #[command]
/// fn command_one() {}
/// #[command]
/// fn command_two() {}
/// fn main() {
///   tauri::Builder::default()
///     .invoke_handler(tauri::generate_handler![command_one, command_two])
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
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

/// Adds a default value `crate::Wry` to the last generic item of a struct/enum
#[doc(hidden)]
#[proc_macro_attribute]
pub fn default_runtime_wry(_attrs: TokenStream, input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  // create a new copy to manipulate for the wry feature flag
  let mut wry = input.clone();
  let wry_runtime = wry
    .generics
    .params
    .last_mut()
    .expect("DefaultRuntime requires at least 1 generic parameter");

  // set the default value of the last generic parameter to crate::wry
  match wry_runtime {
    GenericParam::Type(
      param @ TypeParam {
        eq_token: None,
        default: None,
        ..
      },
    ) => {
      param.eq_token = Some(parse_quote!(=));
      param.default = Some(parse_quote!(crate::Wry));
    }
    _ => {
      panic!("DefaultRuntime requires the last parameter to not have a default value")
    }
  };

  quote::quote!(
    #[cfg(feature = "wry")]
    #wry

    #[cfg(not(feature = "wry"))]
    #input
  )
  .into()
}
