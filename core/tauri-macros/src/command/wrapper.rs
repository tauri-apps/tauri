// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::{ToLowerCamelCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
  ext::IdentExt,
  parse::{Parse, ParseStream},
  parse_macro_input,
  spanned::Spanned,
  FnArg, ItemFn, Lit, Meta, Pat, Token, Visibility,
};

struct WrapperAttributes {
  root: TokenStream2,
  execution_context: ExecutionContext,
  argument_case: ArgumentCase,
}

impl Parse for WrapperAttributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut wrapper_attributes = WrapperAttributes {
      root: quote!(::tauri),
      execution_context: ExecutionContext::Blocking,
      argument_case: ArgumentCase::Camel,
    };

    loop {
      match input.parse::<Meta>() {
        Ok(Meta::List(_)) => {}
        Ok(Meta::NameValue(v)) => {
          if v.path.is_ident("rename_all") {
            if let Lit::Str(s) = v.lit {
              wrapper_attributes.argument_case = match s.value().as_str() {
                "snake_case" => ArgumentCase::Snake,
                "camelCase" => ArgumentCase::Camel,
                _ => {
                  return Err(syn::Error::new(
                    s.span(),
                    "expected \"camelCase\" or \"snake_case\"",
                  ))
                }
              };
            }
          } else if v.path.is_ident("root") {
            if let Lit::Str(s) = v.lit {
              let ident = Ident::new(&s.value(), Span::call_site());
              wrapper_attributes.root = quote!(#ident);
            }
          }
        }
        Ok(Meta::Path(p)) => {
          if p.is_ident("async") {
            wrapper_attributes.execution_context = ExecutionContext::Async;
          } else {
            return Err(syn::Error::new(p.span(), "expected `async`"));
          }
        }
        Err(_e) => {
          break;
        }
      }

      let lookahead = input.lookahead1();
      if lookahead.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      }
    }

    Ok(wrapper_attributes)
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
  let (body, attributes) = syn::parse::<WrapperAttributes>(attributes)
    .map(|mut attrs| {
      if function.sig.asyncness.is_some() {
        attrs.execution_context = ExecutionContext::Async;
      }
      attrs
    })
    .and_then(|attrs| {
      let body = match attrs.execution_context {
        ExecutionContext::Async => body_async(&function, &invoke, &attrs),
        ExecutionContext::Blocking => body_blocking(&function, &invoke, &attrs),
      };
      body.map(|b| (b, Some(attrs)))
    })
    .unwrap_or_else(|e| (syn::Error::into_compile_error(e), None));

  let Invoke { message, resolver } = invoke;

  let root = attributes
    .map(|a| a.root)
    .unwrap_or_else(|| quote!(::tauri));

  // Rely on rust 2018 edition to allow importing a macro from a path.
  quote!(
    #function

    #maybe_macro_export
    #[doc(hidden)]
    macro_rules! #wrapper {
        // double braces because the item is expected to be a block expression
        ($path:path, $invoke:ident) => {{
          #[allow(unused_imports)]
          use #root::command::private::*;
          // prevent warnings when the body is a `compile_error!` or if the command has no arguments
          #[allow(unused_variables)]
          let #root::Invoke { message: #message, resolver: #resolver } = $invoke;

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
fn body_async(
  function: &ItemFn,
  invoke: &Invoke,
  attributes: &WrapperAttributes,
) -> syn::Result<TokenStream2> {
  let Invoke { message, resolver } = invoke;
  parse_args(function, message, attributes).map(|args| {
    quote! {
      #resolver.respond_async_serialized(async move {
        let result = $path(#(#args?),*);
        let kind = (&result).async_kind();
        kind.future(result).await
      });
      return true;
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
  attributes: &WrapperAttributes,
) -> syn::Result<TokenStream2> {
  let Invoke { message, resolver } = invoke;
  let args = parse_args(function, message, attributes)?;

  // the body of a `match` to early return any argument that wasn't successful in parsing.
  let match_body = quote!({
    Ok(arg) => arg,
    Err(err) => { #resolver.invoke_error(err); return true },
  });

  Ok(quote! {
    let result = $path(#(match #args #match_body),*);
    let kind = (&result).blocking_kind();
    kind.block(result, #resolver);
    return true;
  })
}

/// Parse all arguments for the command wrapper to use from the signature of the command function.
fn parse_args(
  function: &ItemFn,
  message: &Ident,
  attributes: &WrapperAttributes,
) -> syn::Result<Vec<TokenStream2>> {
  function
    .sig
    .inputs
    .iter()
    .map(|arg| parse_arg(&function.sig.ident, arg, message, attributes))
    .collect()
}

/// Transform a [`FnArg`] into a command argument.
fn parse_arg(
  command: &Ident,
  arg: &FnArg,
  message: &Ident,
  attributes: &WrapperAttributes,
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

  match attributes.argument_case {
    ArgumentCase::Camel => {
      key = key.to_lower_camel_case();
    }
    ArgumentCase::Snake => {
      key = key.to_snake_case();
    }
  }

  let root = &attributes.root;

  Ok(quote!(#root::command::CommandArg::from_command(
    #root::command::CommandItem {
      name: stringify!(#command),
      key: #key,
      message: &#message,
    }
  )))
}
