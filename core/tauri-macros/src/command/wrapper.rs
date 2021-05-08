// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{
  spanned::Spanned, AttributeArgs, FnArg, Ident, ItemFn, Meta, NestedMeta, Pat, Visibility,
};

enum Asyncness {
  Blocking,
  Async,
}

impl Default for Asyncness {
  fn default() -> Self {
    Self::Blocking
  }
}

#[derive(Default)]
pub struct Attributes {
  asyncness: Asyncness,
}

impl TryFrom<AttributeArgs> for Attributes {
  type Error = syn::Error;

  fn try_from(attributes: AttributeArgs) -> Result<Self, Self::Error> {
    if attributes.is_empty() {
      return Ok(Self::default());
    }

    if attributes.len() == 1 {
      if let NestedMeta::Meta(Meta::Path(path)) = &attributes[0] {
        if path.segments.len() == 1 {
          if let Some(segment) = path.segments.first() {
            if segment.ident == "future" {
              return Ok(Self {
                asyncness: Asyncness::Async,
              });
            }
          }
        }
      }
    }

    Err(syn::Error::new(
      attributes[0].span(),
      "only a single item `async` is currently allowed",
    ))
  }
}

/// The command wrapper created for a function marked with `#[command]`.
pub struct Wrapper {
  function: ItemFn,
  visibility: Visibility,
  maybe_export: TokenStream,
  wrapper: Ident,
  body: syn::Result<TokenStream>,
}

impl Wrapper {
  /// Create a new [`Wrapper`] from the function and the generated code parsed from the function.
  pub fn new(function: ItemFn, attributes: AttributeArgs) -> syn::Result<Self> {
    // macros used with `pub use my_macro;` need to be exported with `#[macro_export]`
    let maybe_export = match &function.vis {
      Visibility::Public(_) => quote!(#[macro_export]),
      _ => Default::default(),
    };

    let mut attributes = Attributes::try_from(attributes)?;

    // force the wrapper to be async if the function is async
    if function.sig.asyncness.is_some() {
      attributes.asyncness = Asyncness::Async
    }

    let visibility = function.vis.clone();
    let wrapper = super::format_command_wrapper(&function.sig.ident);
    let body = match attributes.asyncness {
      Asyncness::Blocking => BlockingBody::try_from(&function).map(|b| b.0),
      Asyncness::Async => AsyncBody::try_from(&function).map(|b| b.0),
    };

    Ok(Self {
      function,
      visibility,
      maybe_export,
      wrapper,
      body,
    })
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
    let body = body.unwrap_or_else(syn::Error::into_compile_error);

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

struct AsyncBody(TokenStream);

impl TryFrom<&ItemFn> for AsyncBody {
  type Error = syn::Error;

  fn try_from(function: &ItemFn) -> syn::Result<Self> {
    // the name of the #[command] function is the name of the command to handle
    let command = function.sig.ident.clone();

    let mut args = Vec::new();
    for param in &function.sig.inputs {
      args.push(parse_arg(&command, param)?);
    }

    // we #[allow(unused_variables)] because a command with no arguments will not use message.
    Ok(Self(quote!(
      use ::tauri::command::private::*;

      #[allow(unused_variables)]
      let ::tauri::Invoke { message, resolver } = $invoke;

      resolver.respond_async(async move {
        let result = $path(#(#args?),*);
        (&result).async_kind().prepare(result).await
      });
    )))
  }
}

/// Body of the wrapper that maps the command parameters into callable arguments from [`Invoke`].
///
/// This is possible because we require the command parameters to be [`CommandArg`] and use type
/// inference to put values generated from that trait into the arguments of the called command.
///
/// [`CommandArg`]: https://docs.rs/tauri/*/tauri/command/trait.CommandArg.html
/// [`Invoke`]: https://docs.rs/tauri/*/tauri/struct.Invoke.html
pub struct BlockingBody(TokenStream);

impl TryFrom<&ItemFn> for BlockingBody {
  type Error = syn::Error;

  fn try_from(function: &ItemFn) -> syn::Result<Self> {
    // the name of the #[command] function is the name of the command to handle
    let command = function.sig.ident.clone();

    let mut args = Vec::new();
    for param in &function.sig.inputs {
      args.push(parse_arg(&command, param)?);
    }

    // the body of a match to early return any argument that wasn't successful in parsing.
    let early_return_error_body = quote!({
      Ok(arg) => arg,
      Err(err) => return resolver.invoke_error(err),
    });

    // we #[allow(unused_variables)] because a command with no arguments will not use message.
    Ok(Self(quote!(
      use ::tauri::command::private::*;

      #[allow(unused_variables)]
      let ::tauri::Invoke { message, resolver } = $invoke;
      let message = ::std::sync::Arc::new(message);
      let result = $path(#(match #args #early_return_error_body),*);

      (&result).blocking_kind().respond(result, resolver);
    )))
  }
}

/// Transform a [`FnArg`] into a command argument. Expects borrowable binding `message` to exist.
fn parse_arg(command: &Ident, arg: &FnArg) -> syn::Result<TokenStream> {
  // we have no use for self arguments
  let mut arg = match arg {
    FnArg::Typed(arg) => arg.pat.as_ref().clone(),
    FnArg::Receiver(arg) => {
      return Err(syn::Error::new(
        arg.span(),
        "unable to use self as a command function parameter",
      ))
    }
  };

  // we only support patterns supported as arguments to a `ItemFn`.
  let key = match &mut arg {
    Pat::Ident(arg) => arg.ident.to_string(),
    Pat::Wild(_) => "_".into(),
    Pat::Struct(s) => super::path_to_command(&mut s.path).ident.to_string(),
    Pat::TupleStruct(s) => super::path_to_command(&mut s.path).ident.to_string(),
    err => {
      return Err(syn::Error::new(
        err.span(),
        "only named, wildcard, struct, and tuple struct arguments allowed",
      ))
    }
  };

  // also catch self arguments that use FnArg::Typed syntax
  if key == "self" {
    return Err(syn::Error::new(
      key.span(),
      "unable to use self as a command function parameter",
    ));
  }

  Ok(quote!(::tauri::command::CommandArg::from_command(
    ::tauri::command::CommandItem {
      name: stringify!(#command),
      key: #key,
      message: &message,
    }
  )))
}
