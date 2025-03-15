// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::env::var;

use heck::{ToLowerCamelCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};
use syn::{
  ext::IdentExt,
  parse::{Parse, ParseStream},
  parse_macro_input,
  punctuated::Punctuated,
  spanned::Spanned,
  Expr, ExprLit, FnArg, ItemFn, Lit, Meta, Pat, Token, Visibility,
};
use tauri_utils::acl::REMOVE_UNUSED_COMMANDS_ENV_VAR;

enum WrapperAttributeKind {
  Meta(Meta),
  Async,
}

impl Parse for WrapperAttributeKind {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    match input.parse::<Meta>() {
      Ok(m) => Ok(Self::Meta(m)),
      Err(e) => match input.parse::<Token![async]>() {
        Ok(_) => Ok(Self::Async),
        Err(_) => Err(e),
      },
    }
  }
}

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

    let attrs = Punctuated::<WrapperAttributeKind, Token![,]>::parse_terminated(input)?;
    for attr in attrs {
      match attr {
        WrapperAttributeKind::Meta(Meta::List(_)) => {
          return Err(syn::Error::new(input.span(), "unexpected list input"));
        }
        WrapperAttributeKind::Meta(Meta::NameValue(v)) => {
          if v.path.is_ident("rename_all") {
            if let Expr::Lit(ExprLit {
              lit: Lit::Str(s),
              attrs: _,
            }) = v.value
            {
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
            if let Expr::Lit(ExprLit {
              lit: Lit::Str(s),
              attrs: _,
            }) = v.value
            {
              let lit = s.value();

              wrapper_attributes.root = if lit == "crate" {
                quote!($crate)
              } else {
                let ident = Ident::new(&lit, Span::call_site());
                quote!(#ident)
              };
            }
          }
        }
        WrapperAttributeKind::Meta(Meta::Path(_)) => {
          return Err(syn::Error::new(
            input.span(),
            "unexpected input, expected one of `rename_all`, `root`, `async`",
          ));
        }
        WrapperAttributeKind::Async => {
          wrapper_attributes.execution_context = ExecutionContext::Async;
        }
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
  acl: Ident,
}

/// Create a new [`Wrapper`] from the function and the generated code parsed from the function.
pub fn wrapper(attributes: TokenStream, item: TokenStream) -> TokenStream {
  let mut attrs = parse_macro_input!(attributes as WrapperAttributes);
  let function = parse_macro_input!(item as ItemFn);
  let wrapper = super::format_command_wrapper(&function.sig.ident);
  let visibility = &function.vis;

  if function.sig.asyncness.is_some() {
    attrs.execution_context = ExecutionContext::Async;
  }

  // macros used with `pub use my_macro;` need to be exported with `#[macro_export]`
  let maybe_macro_export = match &function.vis {
    Visibility::Public(_) | Visibility::Restricted(_) => quote!(#[macro_export]),
    _ => TokenStream2::default(),
  };

  let invoke = Invoke {
    message: format_ident!("__tauri_message__"),
    resolver: format_ident!("__tauri_resolver__"),
    acl: format_ident!("__tauri_acl__"),
  };

  // Tauri currently doesn't support async commands that take a reference as input and don't return
  // a result. See: https://github.com/tauri-apps/tauri/issues/2533
  //
  // For now, we provide an informative error message to the user in that case. Once #2533 is
  // resolved, this check can be removed.
  let mut async_command_check = TokenStream2::new();
  if function.sig.asyncness.is_some() {
    // This check won't catch all possible problems but it should catch the most common ones.
    let mut ref_argument_span = None;

    for arg in &function.sig.inputs {
      if let syn::FnArg::Typed(pat) = arg {
        match &*pat.ty {
          syn::Type::Reference(_) => {
            ref_argument_span = Some(pat.span());
          }
          syn::Type::Path(path) => {
            // Check if the type contains a lifetime argument
            let last = path.path.segments.last().unwrap();
            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
              if args
                .args
                .iter()
                .any(|arg| matches!(arg, syn::GenericArgument::Lifetime(_)))
              {
                ref_argument_span = Some(pat.span());
              }
            }
          }
          _ => {}
        }

        if let Some(span) = ref_argument_span {
          if let syn::ReturnType::Type(_, return_type) = &function.sig.output {
            // To check if the return type is `Result` we require it to check a trait that is
            // only implemented by `Result`. That way we don't exclude renamed result types
            // which we wouldn't otherwise be able to detect purely from the token stream.
            // The "error message" displayed to the user is simply the trait name.
            //
            // TODO: remove this check once our MSRV is high enough
            let diagnostic = if is_rustc_at_least(1, 78) {
              quote!(#[diagnostic::on_unimplemented(message = "async commands that contain references as inputs must return a `Result`")])
            } else {
              quote!()
            };

            async_command_check = quote_spanned! {return_type.span() =>
              #[allow(unreachable_code, clippy::diverging_sub_expression)]
              const _: () = if false {
                #diagnostic
                trait AsyncCommandMustReturnResult {}
                impl<A, B> AsyncCommandMustReturnResult for ::std::result::Result<A, B> {}
                let _check: #return_type = unreachable!();
                let _: &dyn AsyncCommandMustReturnResult = &_check;
              };
            };
          } else {
            return quote_spanned! {
              span => compile_error!("async commands that contain references as inputs must return a `Result`");
            }.into();
          }
        }
      }
    }
  }

  let plugin_name = var("CARGO_PKG_NAME")
    .expect("missing `CARGO_PKG_NAME` environment variable")
    .strip_prefix("tauri-plugin-")
    .map(|name| quote!(::core::option::Option::Some(#name)))
    .unwrap_or_else(|| quote!(::core::option::Option::None));

  let body = match attrs.execution_context {
    ExecutionContext::Async => body_async(&plugin_name, &function, &invoke, &attrs)
      .unwrap_or_else(syn::Error::into_compile_error),
    ExecutionContext::Blocking => body_blocking(&plugin_name, &function, &invoke, &attrs)
      .unwrap_or_else(syn::Error::into_compile_error),
  };

  let Invoke {
    message,
    resolver,
    acl,
  } = invoke;

  let root = attrs.root;

  let kind = match attrs.execution_context {
    ExecutionContext::Async if function.sig.asyncness.is_none() => "sync_threadpool",
    ExecutionContext::Async => "async",
    ExecutionContext::Blocking => "sync",
  };

  let loc = function.span().start();
  let line = loc.line;
  let col = loc.column;

  let maybe_span = if cfg!(feature = "tracing") {
    quote!({
      let _span = tracing::debug_span!(
        "ipc::request::handler",
        cmd = #message.command(),
        kind = #kind,
        loc.line = #line,
        loc.col = #col,
        is_internal = false,
      )
      .entered();
    })
  } else {
    quote!()
  };

  // Allow this to be unused when we're building with `build > removeUnusedCommands` for dead code elimination
  let maybe_allow_unused = if var(REMOVE_UNUSED_COMMANDS_ENV_VAR).is_ok() {
    quote!(#[allow(unused)])
  } else {
    TokenStream2::default()
  };

  // Rely on rust 2018 edition to allow importing a macro from a path.
  quote!(
    #async_command_check

    #maybe_allow_unused
    #function

    #maybe_allow_unused
    #maybe_macro_export
    #[doc(hidden)]
    macro_rules! #wrapper {
        // double braces because the item is expected to be a block expression
        ($path:path, $invoke:ident) => {{
          #[allow(unused_imports)]
          use #root::ipc::private::*;
          // prevent warnings when the body is a `compile_error!` or if the command has no arguments
          #[allow(unused_variables)]
          let #root::ipc::Invoke { message: #message, resolver: #resolver, acl: #acl } = $invoke;

          #maybe_span

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
  plugin_name: &TokenStream2,
  function: &ItemFn,
  invoke: &Invoke,
  attributes: &WrapperAttributes,
) -> syn::Result<TokenStream2> {
  let Invoke {
    message,
    resolver,
    acl,
  } = invoke;
  parse_args(plugin_name, function, message, acl, attributes).map(|args| {
    #[cfg(feature = "tracing")]
    quote! {
      use tracing::Instrument;

      let span = tracing::debug_span!("ipc::request::run");
      #resolver.respond_async_serialized(async move {
        let result = $path(#(#args?),*);
        let kind = (&result).async_kind();
        kind.future(result).await
      }
      .instrument(span));
      return true;
    }

    #[cfg(not(feature = "tracing"))]
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
  plugin_name: &TokenStream2,
  function: &ItemFn,
  invoke: &Invoke,
  attributes: &WrapperAttributes,
) -> syn::Result<TokenStream2> {
  let Invoke {
    message,
    resolver,
    acl,
  } = invoke;
  let args = parse_args(plugin_name, function, message, acl, attributes)?;

  // the body of a `match` to early return any argument that wasn't successful in parsing.
  let match_body = quote!({
    Ok(arg) => arg,
    Err(err) => { #resolver.invoke_error(err); return true },
  });

  let maybe_span = if cfg!(feature = "tracing") {
    quote!(let _span = tracing::debug_span!("ipc::request::run").entered();)
  } else {
    quote!()
  };

  Ok(quote! {
    #maybe_span
    let result = $path(#(match #args #match_body),*);
    let kind = (&result).blocking_kind();
    kind.block(result, #resolver);
    return true;
  })
}

/// Parse all arguments for the command wrapper to use from the signature of the command function.
fn parse_args(
  plugin_name: &TokenStream2,
  function: &ItemFn,
  message: &Ident,
  acl: &Ident,
  attributes: &WrapperAttributes,
) -> syn::Result<Vec<TokenStream2>> {
  function
    .sig
    .inputs
    .iter()
    .map(|arg| {
      parse_arg(
        plugin_name,
        &function.sig.ident,
        arg,
        message,
        acl,
        attributes,
      )
    })
    .collect()
}

/// Transform a [`FnArg`] into a command argument.
fn parse_arg(
  plugin_name: &TokenStream2,
  command: &Ident,
  arg: &FnArg,
  message: &Ident,
  acl: &Ident,
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

  Ok(quote!(#root::ipc::CommandArg::from_command(
    #root::ipc::CommandItem {
      plugin: #plugin_name,
      name: stringify!(#command),
      key: #key,
      message: &#message,
      acl: &#acl,
    }
  )))
}

fn is_rustc_at_least(major: u32, minor: u32) -> bool {
  let version = rustc_version();
  version.0 >= major && version.1 >= minor
}

fn rustc_version() -> (u32, u32) {
  cross_command("rustc")
    .arg("-V")
    .output()
    .ok()
    .and_then(|o| {
      let version = String::from_utf8_lossy(&o.stdout)
        .trim()
        .split(' ')
        .nth(1)
        .unwrap_or_default()
        .split('.')
        .take(2)
        .flat_map(|p| p.parse::<u32>().ok())
        .collect::<Vec<_>>();
      version
        .first()
        .and_then(|major| version.get(1).map(|minor| (*major, *minor)))
    })
    .unwrap_or((1, 0))
}

fn cross_command(bin: &str) -> std::process::Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = std::process::Command::new(bin);
  cmd
}
