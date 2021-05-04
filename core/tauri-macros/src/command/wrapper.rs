// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::convert::TryFrom;
use syn::{spanned::Spanned, FnArg, Ident, ItemFn, Pat, ReturnType, Type, Visibility};

/// The command wrapper created for a function marked with `#[command]`.
pub struct Wrapper {
  function: ItemFn,
  visibility: Visibility,
  maybe_export: TokenStream,
  wrapper: Ident,
  body: syn::Result<WrapperBody>,
}

impl Wrapper {
  /// Create a new [`Wrapper`] from the function and the generated code parsed from the function.
  pub fn new(function: ItemFn, body: syn::Result<WrapperBody>) -> Self {
    // macros used with `pub use my_macro;` need to be exported with `#[macro_export]`
    let maybe_export = match &function.vis {
      Visibility::Public(_) => quote!(#[macro_export]),
      _ => Default::default(),
    };

    let visibility = function.vis.clone();
    let wrapper = super::format_command_wrapper(&function.sig.ident);

    Self {
      function,
      visibility,
      maybe_export,
      wrapper,
      body,
    }
  }
}

impl From<Wrapper> for proc_macro::TokenStream {
  fn from(
    Wrapper {
      function,
      maybe_export,
      wrapper,
      body,
      visibility,
    }: Wrapper,
  ) -> Self {
    // either use the successful body or a `compile_error!` of the error occurred while parsing it.
    let body = body
      .as_ref()
      .map(ToTokens::to_token_stream)
      .unwrap_or_else(syn::Error::to_compile_error);

    // we `use` the macro so that other modules can resolve the with the same path as the function.
    // this is dependent on rust 2018 edition.
    quote!(
      #function
      #maybe_export
      macro_rules! #wrapper { ($path:path, $invoke:ident) => {{ #body }}; }
      #visibility use #wrapper;
    )
    .into()
  }
}

/// Body of the wrapper that maps the command parameters into callable arguments from [`Invoke`].
///
/// This is possible because we require the command parameters to be [`CommandArg`] and use type
/// inference to put values generated from that trait into the arguments of the called command.
///
/// [`CommandArg`]: https://docs.rs/tauri/*/tauri/command/trait.CommandArg.html
/// [`Invoke`]: https://docs.rs/tauri/*/tauri/struct.Invoke.html
pub struct WrapperBody(TokenStream);

impl TryFrom<&ItemFn> for WrapperBody {
  type Error = syn::Error;

  fn try_from(function: &ItemFn) -> syn::Result<Self> {
    // the name of the #[command] function is the name of the command to handle
    let command = function.sig.ident.clone();

    // automatically append await when the #[command] function is async
    let maybe_await = match function.sig.asyncness {
      Some(_) => quote!(.await),
      None => Default::default(),
    };

    // todo: detect command return types automatically like params, removes parsing type name
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

    let mut args = Vec::new();
    for param in &function.sig.inputs {
      args.push(parse_arg(&command, param)?);
    }

    // todo: change this to automatically detect result returns (see above result todo)
    // if the command handler returns a Result,
    // we just map the values to the ones expected by Tauri
    // otherwise we wrap it with an `Ok()`, converting the return value to tauri::InvokeResponse
    // note that all types must implement `serde::Serialize`.
    let result = if returns_result {
      quote! {
        let result = $path(#(#args?),*);
        ::core::result::Result::Ok(result #maybe_await?)
      }
    } else {
      quote! {
        let result = $path(#(#args?),*);
        ::core::result::Result::<_, ::tauri::InvokeError>::Ok(result #maybe_await)
      }
    };

    Ok(Self(result))
  }
}

impl ToTokens for WrapperBody {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let body = &self.0;

    // we #[allow(unused_variables)] because a command with no arguments will not use message.
    tokens.append_all(quote!(
      #[allow(unused_variables)]
      let ::tauri::Invoke { message, resolver } = $invoke;
      resolver.respond_async(async move { #body });
    ))
  }
}

/// Transform a [`FnArg`] into a command argument. Expects borrowable binding `message` to exist.
fn parse_arg(command: &Ident, arg: &FnArg) -> syn::Result<TokenStream> {
  // we have no use for self arguments
  let arg = match arg {
    FnArg::Typed(arg) => arg.pat.as_ref(),
    FnArg::Receiver(arg) => {
      return Err(syn::Error::new(
        arg.span(),
        "unable to use self as a command function parameter",
      ))
    }
  };

  // we only have use for ident typed patterns
  let arg = match arg {
    Pat::Ident(arg) => &arg.ident,
    err => {
      return Err(syn::Error::new(
        err.span(),
        "command parameters expected to be a typed pattern",
      ))
    }
  };

  // also catch self arguments that use FnArg::Typed syntax
  if arg == "self" {
    return Err(syn::Error::new(
      arg.span(),
      "unable to use self as a command function parameter",
    ));
  }

  Ok(quote!(::tauri::command::CommandArg::from_command(
    ::tauri::command::CommandItem {
      name: stringify!(#command),
      key: stringify!(#arg),
      message: &message,
    }
  )))
}
