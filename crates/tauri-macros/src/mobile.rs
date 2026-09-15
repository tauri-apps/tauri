// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::env::var;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

fn get_env_var(name: &str, error: &mut Option<TokenStream2>, function: &ItemFn) -> TokenStream2 {
  match var(name) {
    Ok(value) => {
      let ident = format_ident!("{value}");
      quote!(#ident)
    }
    Err(_) => {
      error.replace(
        syn::Error::new(
          function.span(),
          format!("`{name}` env var not set, do you have a build script with tauri-build?",),
        )
        .into_compile_error(),
      );
      quote!()
    }
  }
}

pub fn entry_point(_attributes: TokenStream, item: TokenStream) -> TokenStream {
  let function = parse_macro_input!(item as ItemFn);
  let function_name = function.sig.ident.clone();

  let mut error = None;
  let domain = get_env_var("TAURI_ANDROID_PACKAGE_NAME_PREFIX", &mut error, &function);
  let app_name = get_env_var("TAURI_ANDROID_PACKAGE_NAME_APP_NAME", &mut error, &function);

  let (wrapper, wrapper_name) = if function.sig.asyncness.is_some() {
    let wrapper_name = syn::Ident::new(&format!("{function_name}_wrapper"), function_name.span());
    (
      quote! {
        #function

        fn #wrapper_name() {
          ::tauri::async_runtime::block_on(#function_name());
        }
      },
      wrapper_name,
    )
  } else {
    (
      quote! {
        #function
      },
      function_name,
    )
  };

  if let Some(e) = error {
    quote!(#e).into()
  } else {
    quote!(
      fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
          Ok(t) => t,
          Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
          }
        }
      }

      #wrapper

      fn _start_app() {
        #[cfg(target_os = "ios")]
        ::tauri::log_stdout();
        #[cfg(target_os = "android")]
        {
          ::tauri::android_binding!(#domain, #app_name, _start_app, ::tauri::wry);
        }
        stop_unwind(#wrapper_name);
      }

      // be careful when renaming this, the `start_app` symbol is checked by the CLI
      #[cfg(not(target_os = "android"))]
      #[no_mangle]
      #[inline(never)]
      pub extern "C" fn start_app() {
        _start_app()
      }
    )
    .into()
  }
}
