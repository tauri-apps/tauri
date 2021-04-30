// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
  parse::Parser, punctuated::Punctuated, FnArg, GenericArgument, Ident, ItemFn, Meta, NestedMeta,
  Pat, Path, PathArguments, ReturnType, Token, Type, Visibility,
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

pub fn generate_command(attrs: Vec<NestedMeta>, function: ItemFn) -> TokenStream {
  // Check if "with_window" attr was passed to macro
  let with_window = attrs.iter().any(|a| {
    if let NestedMeta::Meta(Meta::Path(path)) = a {
      path
        .get_ident()
        .map(|i| *i == "with_window")
        .unwrap_or(false)
    } else {
      false
    }
  });

  let fn_name = function.sig.ident.clone();
  let fn_name_str = fn_name.to_string();
  let fn_wrapper = format_ident!("{}_wrapper", fn_name);
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
  let mut call_arguments = Vec::new();

  for (i, param) in function.sig.inputs.clone().into_iter().enumerate() {
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

    if i == 0 && with_window {
      call_arguments.push(quote!(_window));
      continue;
    }

    let arg_name_ = arg_name.clone().unwrap();
    let arg_type =
      arg_type.unwrap_or_else(|| panic!("Invalid type for arg \"{}\"", arg_name.unwrap()));

    let mut path_as_string = String::new();
    for segment in &arg_type.segments {
      path_as_string.push_str(&segment.ident.to_string());
      path_as_string.push_str("::");
    }

    if ["State::", "tauri::State::"].contains(&path_as_string.as_str()) {
      let last_segment = match arg_type.segments.last() {
        Some(last) => last,
        None => return err(function, "found a type path without any segments (how?)"),
      };
      if let PathArguments::AngleBracketed(angle) = &last_segment.arguments {
        if let Some(GenericArgument::Type(ty)) = angle.args.last() {
          call_arguments.push(quote!(state_manager.get::<#ty>()));
          continue;
        }
      }
    }

    invoke_arg_names.push(arg_name_.clone());
    invoke_arg_types.push(arg_type);
    call_arguments.push(quote!(parsed_args.#arg_name_));
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
      match #fn_name(#(#call_arguments),*)#await_maybe {
        Ok(value) => ::core::result::Result::Ok(value),
        Err(e) => ::core::result::Result::Err(e),
      }
    }
  } else {
    quote! { ::core::result::Result::<_, ()>::Ok(#fn_name(#(#call_arguments),*)#await_maybe) }
  };

  quote! {
    #function
    pub fn #fn_wrapper<P: ::tauri::Params>(message: ::tauri::InvokeMessage<P>, state_manager: ::std::sync::Arc<::tauri::StateManager>) {
      #[derive(::serde::Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct ParsedArgs {
        #(#invoke_arg_names: #invoke_arg_types),*
      }
      let _window = message.window();
      match ::serde_json::from_value::<ParsedArgs>(message.payload()) {
        Ok(parsed_args) => message.respond_async(async move {
          #return_value
        }),
        Err(e) => message.reject(::tauri::Error::InvalidArgs(#fn_name_str, e).to_string()),
      }
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
    move |message, state_manager| {
      let cmd = message.command().to_string();
      match cmd.as_str() {
        #(stringify!(#fn_names) => #fn_wrappers(message, state_manager),)*
        _ => {
          message.reject(format!("command {} not found", cmd))
        },
      }
    }
  }
}
