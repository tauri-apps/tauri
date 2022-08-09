// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
  ext::IdentExt,
  parse::{Parse, ParseStream},
  parse_macro_input,
  spanned::Spanned,
  Expr, ExprAssign, FnArg, Ident, ItemFn, Lit, Pat, Token, Visibility,
};

struct WrapperAttributes {
  execution_context: ExecutionContext,
  argument_case: ArgumentCase,
}

impl Parse for WrapperAttributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let execution_context = input
      .parse::<Token![async]>()
      .map(|_| ExecutionContext::Async)
      .unwrap_or(ExecutionContext::Blocking);

    let _ = input.parse::<Token![,]>();

    let mut argument_case = ArgumentCase::Camel;
    if let Ok(assign) = input.parse::<ExprAssign>() {
      if let Expr::Path(i) = *assign.left {
        if i.path.is_ident("rename_all") {
          if let Expr::Lit(l) = *assign.right {
            if let Lit::Str(s) = l.lit {
              argument_case = match s.value().as_str() {
                "snake_case" => ArgumentCase::Snake,
                "camelCase" => ArgumentCase::Camel,
                _ => {
                  return Err(syn::Error::new(
                    i.span(),
                    "expected \"camelCase\" or \"snake_case\"",
                  ))
                }
              };
            }
          }
        }
      }
    }

    Ok(Self {
      execution_context,
      argument_case,
    })
  }
}

/// The execution context of the command.
enum ExecutionContext {
  Async,
  Blocking,
}

/// The case of each argument name.
#[derive(Copy, Clone)]
enum ArgumentCase {
  Snake,
  Camel,
}

/// The bindings we attach to `tauri::Invoke`.
struct Invoke {
  message: Ident,
  resolver: Ident,
}

/// Create a new [`Wrapper`] from the function and the generated code parsed from the function.
pub fn wrapper(attributes: TokenStream, item: TokenStream) -> TokenStream {
  let function = parse_macro_input!(item as ItemFn);
  let wrapper = super::format_command_wrapper(&function.sig.ident);
  let visibility = &function.vis;

  // macros used with `pub use my_macro;` need to be exported with `#[macro_export]`
  let maybe_macro_export = match &function.vis {
    Visibility::Public(_) => quote!(#[macro_export]),
    _ => Default::default(),
  };

  let invoke = Invoke {
    message: format_ident!("__tauri_message__"),
    resolver: format_ident!("__tauri_resolver__"),
  };

  // body to the command wrapper or a `compile_error!` of an error occurred while parsing it.
  let body = syn::parse::<WrapperAttributes>(attributes)
    .and_then(|mut attrs| {
      if function.sig.asyncness.is_some() {
        attrs.execution_context = ExecutionContext::Async;
      }
      Ok(attrs)
    })
    .and_then(|attrs| match attrs.execution_context {
      ExecutionContext::Async => body_async(&function, &invoke, attrs.argument_case),
      ExecutionContext::Blocking => body_blocking(&function, &invoke, attrs.argument_case),
    })
    .unwrap_or_else(syn::Error::into_compile_error);

  let Invoke { message, resolver } = invoke;

  // Rely on rust 2018 edition to allow importing a macro from a path.
  quote!(
    #function

    #maybe_macro_export
    #[doc(hidden)]
    macro_rules! #wrapper {
        // double braces because the item is expected to be a block expression
        ($path:path, $invoke:ident) => {{
          #[allow(unused_imports)]
          use ::tauri::command::private::*;
          // prevent warnings when the body is a `compile_error!` or if the command has no arguments
          #[allow(unused_variables)]
          let ::tauri::Invoke { message: #message, resolver: #resolver } = $invoke;

          #body
      }};
    }

    // allow the macro to be resolved with the same path as the command function
    #[allow(unused_imports)]
    #visibility use #wrapper;
  )
  .into()
}

/// Generates an asynchronous command response from the arguments and return value of a function.
///
/// See the [`tauri::command`] module for all the items and traits that make this possible.
///
/// [`tauri::command`]: https://docs.rs/tauri/*/tauri/runtime/index.html
fn body_async(function: &ItemFn, invoke: &Invoke, case: ArgumentCase) -> syn::Result<TokenStream2> {
  let Invoke { message, resolver } = invoke;
  parse_args(function, message, case).map(|args| {
    quote! {
      #resolver.respond_async_serialized(async move {
        let result = $path(#(#args?),*);
        let kind = (&result).async_kind();
        kind.future(result).await
      });
    }
  })
}

/// Generates a blocking command response from the arguments and return value of a function.
///
/// See the [`tauri::command`] module for all the items and traits that make this possible.
///
/// [`tauri::command`]: https://docs.rs/tauri/*/tauri/runtime/index.html
fn body_blocking(
  function: &ItemFn,
  invoke: &Invoke,
  case: ArgumentCase,
) -> syn::Result<TokenStream2> {
  let Invoke { message, resolver } = invoke;
  let args = parse_args(function, message, case)?;

  // the body of a `match` to early return any argument that wasn't successful in parsing.
  let match_body = quote!({
    Ok(arg) => arg,
    Err(err) => return #resolver.invoke_error(err),
  });

  Ok(quote! {
    let result = $path(#(match #args #match_body),*);
    let kind = (&result).blocking_kind();
    kind.block(result, #resolver);
  })
}

/// Parse all arguments for the command wrapper to use from the signature of the command function.
fn parse_args(
  function: &ItemFn,
  message: &Ident,
  case: ArgumentCase,
) -> syn::Result<Vec<TokenStream2>> {
  function
    .sig
    .inputs
    .iter()
    .map(|arg| parse_arg(&function.sig.ident, arg, message, case))
    .collect()
}

/// Transform a [`FnArg`] into a command argument.
fn parse_arg(
  command: &Ident,
  arg: &FnArg,
  message: &Ident,
  case: ArgumentCase,
) -> syn::Result<TokenStream2> {
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

  // we only support patterns that allow us to extract some sort of keyed identifier
  let mut key = match &mut arg {
    Pat::Ident(arg) => arg.ident.unraw().to_string(),
    Pat::Wild(_) => "".into(), // we always convert to camelCase, so "_" will end up empty anyways
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

  if let ArgumentCase::Camel = case {
    // snake_case -> camelCase
    if key.as_str().contains('_') {
      key = snake_case_to_camel_case(key.as_str());
    }
  }

  Ok(quote!(::tauri::command::CommandArg::from_command(
    ::tauri::command::CommandItem {
      name: stringify!(#command),
      key: #key,
      message: &#message,
    }
  )))
}

/// Convert a snake_case string into camelCase, no underscores will be left.
fn snake_case_to_camel_case(key: &str) -> String {
  let mut camel = String::with_capacity(key.len());
  let mut to_upper = false;

  for c in key.chars() {
    match c {
      '_' => to_upper = true,
      c if std::mem::take(&mut to_upper) => camel.push(c.to_ascii_uppercase()),
      c => camel.push(c),
    }
  }

  camel
}
