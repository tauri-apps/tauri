use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::env::var;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

fn get_env_var<R: FnOnce(String) -> String>(
  name: &str,
  replacer: R,
  error: &mut Option<TokenStream2>,
  function: &ItemFn,
) -> TokenStream2 {
  match var(name) {
    Ok(value) => {
      let ident = format_ident!("{}", replacer(value));
      quote!(#ident)
    }
    Err(_) => {
      error.replace(
        syn::Error::new(
          function.span(),
          format!(
            "`{}` env var not set, do you have a build script with tauri-build?",
            name,
          ),
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
  let domain = get_env_var("TAURI_ANDROID_DOMAIN", |r| r, &mut error, &function);
  let app_name = get_env_var(
    "CARGO_PKG_NAME",
    |r| r.replace('_', "_1"),
    &mut error,
    &function,
  );

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

      #function

      fn _start_app() {
        #[cfg(target_os = "android")]
        {
          use ::tauri::paste;
          ::tauri::wry_android_binding!(#domain, #app_name, _start_app, ::tauri::wry);
        }
        stop_unwind(#function_name);
      }

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
