// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn entry_point(_attributes: TokenStream, item: TokenStream) -> TokenStream {
  let mut function = parse_macro_input!(item as ItemFn);
  let original_block = function.block.clone();

  let has_return_type = function.sig.output != syn::ReturnType::Default;
  let process_check = if has_return_type {
    quote! {
      // Check if this is the browser process
      let is_browser_process = !std::env::args()
        .any(|arg| arg.starts_with("--type="));

      if !is_browser_process {
        ::tauri::run_cef_helper_process();
        return Default::default();
      }
    }
  } else {
    quote! {
      // Check if this is the browser process
      let is_browser_process = !std::env::args()
        .any(|arg| arg.starts_with("--type="));

      if !is_browser_process {
        ::tauri::run_cef_helper_process();
        return;
      }
    }
  };

  function.block = syn::parse2(quote! {
    {
      #process_check
      #original_block
    }
  })
  .expect("Failed to parse block");

  quote!(#function).into()
}
