// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
  parse::Parser, punctuated::Punctuated, FnArg, GenericArgument, Ident, ItemFn, Pat, Path,
  PathArguments, ReturnType, Token, Type, Visibility,
};

fn fn_wrapper(function: &ItemFn) -> (&Visibility, Ident) {
  (
    &function.vis,
    format_ident!("{}_wrapper", function.sig.ident),
  )
}

fn err(function: ItemFn, error_message: &str) -> TokenStream {
  let (vis, wrap) = fn_wrapper(&function);
  quote! {
    #function

    #vis fn #wrap<P: ::tauri::Params>(_message: ::tauri::InvokeMessage<P>) {
      compile_error!(#error_message);
      unimplemented!()
    }
  }
}

pub fn generate_command(function: ItemFn) -> TokenStream {
  /*let mut params = quote!(::tauri::Params);
  let mut fail = true;
  if let FnArg::Typed(pat) = window {
    if let Type::Path(ty) = &*pat.ty {
      let last = match ty.path.segments.last() {
        Some(last) => last,
        None => {
          return err(
            function,
            "found a type path (expected to be window) without any segments (how?)",
          )
        }
      };

      let angle = match &last.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => {
          return err(
            function,
            "type path (expected to be window) needs to have an angled generic argument",
          )
        }
      };

      if angle.args.len() != 1 {
        return err(
          function,
          "type path (expected to be window) needs to have exactly one generic argument",
        );
      }

      if let Some(GenericArgument::Type(Type::ImplTrait(ty))) = angle.args.first() {
        if ty.bounds.len() > 1 {
          return err(
              function,
              "only a single bound is allowed for the window in #[command(with_window)], ::tauri::Params"
            );
        }

        if let Some(bound) = ty.bounds.first() {
          params = bound.to_token_stream();
          fail = false;
        };
      }
    }
  }

  if fail {
    return err(
      function,
      "only impl trait is supported for now... this should not have gotten merged",
    );
  }*/

  let fn_name = function.sig.ident.clone();
  let fn_name_str = fn_name.to_string();
  let (vis, fn_wrapper) = fn_wrapper(&function);
  let returns_result = match function.sig.output {
    ReturnType::Type(_, ref ty) => match &**ty {
      Type::Path(type_path) => {
        type_path
          .path
          .segments
          .first()
          .map(|seg| seg.ident.to_string())
          == Some("Result".to_string())
      }
      _ => false,
    },
    ReturnType::Default => false,
  };

  let mut invoke_arg_names: Vec<Ident> = Default::default();
  let mut invoke_arg_types: Vec<Path> = Default::default();
  let mut invoke_args: TokenStream = Default::default();

  for param in &function.sig.inputs {
    let mut arg_name = None;
    let mut arg_type = None;
    if let FnArg::Typed(arg) = param {
      if let Pat::Ident(ident) = arg.pat.as_ref() {
        arg_name = Some(ident.ident.clone());
      }
      if let Type::Path(path) = arg.ty.as_ref() {
        arg_type = Some(path.path.clone());
      }
    }

    let arg_name_ = arg_name.unwrap();
    let arg_name_s = arg_name_.to_string();

    let arg_type = match arg_type {
      Some(arg_type) => arg_type,
      None => {
        return err(
          function.clone(),
          &format!("invalid type for arg: {}", arg_name_),
        )
      }
    };

    let item = quote!(::tauri::command::CommandItem {
      name: #fn_name_str,
      key: #arg_name_s,
      message: &__message,
    });

    invoke_args.append_all(quote!(let #arg_name_ = <#arg_type>::from_command(#item)?;));
    invoke_arg_names.push(arg_name_);
    invoke_arg_types.push(arg_type);
  }

  let await_maybe = if function.sig.asyncness.is_some() {
    quote!(.await)
  } else {
    quote!()
  };

  // if the command handler returns a Result,
  // we just map the values to the ones expected by Tauri
  // otherwise we wrap it with an `Ok()`, converting the return value to tauri::InvokeResponse
  // note that all types must implement `serde::Serialize`.
  let return_value = if returns_result {
    quote!(::core::result::Result::Ok(#fn_name(#(#invoke_arg_names),*)#await_maybe?))
  } else {
    quote! { ::core::result::Result::<_, ::tauri::InvokeError>::Ok(#fn_name(#(#invoke_arg_names),*)#await_maybe) }
  };

  // double underscore prefix temporary until underlying scoping issue is fixed (planned)
  quote! {
    #function
    #vis fn #fn_wrapper(invoke: ::tauri::Invoke<impl ::tauri::Params>) {
      use ::tauri::command::CommandArg;
      let ::tauri::Invoke { message: __message, resolver: __resolver } = invoke;
      __resolver.respond_async(async move {
        #invoke_args
        #return_value
      })
    }
  }
}

pub fn generate_handler(item: proc_macro::TokenStream) -> TokenStream {
  // Get paths of functions passed to macro
  let paths = <Punctuated<Path, Token![,]>>::parse_terminated
    .parse(item)
    .expect("generate_handler!: Failed to parse list of command functions");

  // Get names of functions, used for match statement
  let fn_names = paths
    .iter()
    .map(|p| p.segments.last().unwrap().ident.clone());

  // Get paths to wrapper functions
  let fn_wrappers = paths.iter().map(|func| {
    let mut func = func.clone();
    let mut last_segment = func.segments.last_mut().unwrap();
    last_segment.ident = format_ident!("{}_wrapper", last_segment.ident);
    func
  });

  quote! {
    move |invoke| {
      let cmd = invoke.message.command();
      match cmd {
        #(stringify!(#fn_names) => #fn_wrappers(invoke),)*
        _ => {
          invoke.resolver.reject(format!("command {} not found", cmd))
        },
      }
    }
  }
}
