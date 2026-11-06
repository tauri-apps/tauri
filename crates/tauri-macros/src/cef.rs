// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn entry_point(_attributes: TokenStream, item: TokenStream) -> TokenStream {
  let mut function = parse_macro_input!(item as ItemFn);
  let original_block = function.block.clone();
  let function_name = function.sig.ident.clone();

  let has_return_type = function.sig.output != syn::ReturnType::Default;
  let process_check = if has_return_type {
    quote! {
      // Check if this is the browser process
      let is_browser_process = !std::env::args()
        .any(|arg| arg.starts_with("--type="));

      if !is_browser_process {
        ::tauri_runtime_cef::run_cef_helper_process();
        return Default::default();
      }
    }
  } else {
    quote! {
      // Check if this is the browser process
      let is_browser_process = !std::env::args()
        .any(|arg| arg.starts_with("--type="));

      if !is_browser_process {
        ::tauri_runtime_cef::run_cef_helper_process();
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

  // CEF's Windows bootstrap executable loads the application as a DLL and calls
  // `RunWinMain` in every process it starts (browser, renderer, GPU, ...), handing it
  // the sandbox information object that makes Chromium's sandbox work. Record the
  // pointer before the wrapped function runs so the runtime can pass it to
  // `CefExecuteProcess` and `CefInitialize`.
  //
  // A binary cannot be loaded by the bootstrap, and a library linked into one must not
  // define a second `RunWinMain` next to the binary's. Both cases are covered by only
  // exporting from a function that is not the executable's `main`: the documented
  // Windows bootstrap entry point is a `pub fn run` in a `cdylib` library, while an
  // application that keeps the attribute on `main` stays an ordinary executable.
  let windows_bootstrap_entry = if function_name == "main" {
    quote! {}
  } else {
    quote! {
      #[cfg(windows)]
      #[doc(hidden)]
      #[unsafe(no_mangle)]
      pub extern "C" fn RunWinMain(
        _instance: *mut ::core::ffi::c_void,
        _command_line: *mut u16,
        _command_show: ::core::ffi::c_int,
        sandbox_info: *mut u8,
        _version_info: *mut ::core::ffi::c_void,
      ) -> ::core::ffi::c_int {
        ::tauri_runtime_cef::set_windows_sandbox_info(sandbox_info);
        let _ = #function_name();
        0
      }
    }
  };

  quote!(#function #windows_bootstrap_entry).into()
}
