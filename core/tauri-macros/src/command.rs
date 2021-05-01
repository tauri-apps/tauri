// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::{
  parse::Parser, punctuated::Punctuated, FnArg, Ident, ItemFn, Pat, Path, ReturnType, Token, Type,
  Visibility,
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

    let arg_name_ = arg_name.clone().unwrap();
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

    invoke_args.append_all(quote! {
      let #arg_name_ = match <#arg_type>::from_command(#fn_name_str, #arg_name_s, &message) {
        Ok(value) => value,
        Err(e) => return tauri::InvokeResponse::Err(::tauri::Error::InvalidArgs(#fn_name_str, e).to_string())
      };
    });
    invoke_arg_names.push(arg_name_.clone());
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
    quote! {
      match #fn_name(#(#invoke_arg_names),*)#await_maybe {
        Ok(value) => ::core::result::Result::Ok(value).into(),
        Err(e) => ::core::result::Result::Err(e).into(),
      }
    }
  } else {
    quote! { ::core::result::Result::<_, ()>::Ok(#fn_name(#(#invoke_arg_names),*)#await_maybe).into() }
  };

  quote! {
    #function
    #vis fn #fn_wrapper<P: ::tauri::Params>(message: ::tauri::InvokeMessage<P>, resolver: ::tauri::InvokeResolver<P>) {
      use ::tauri::command::FromCommand;
      resolver.respond_async(async move {
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
    move |message, resolver| {
      let cmd = message.command().to_string();
      match cmd.as_str() {
        #(stringify!(#fn_names) => #fn_wrappers(message, resolver),)*
        _ => {
          resolver.reject(format!("command {} not found", cmd))
        },
      }
    }
  }
}
