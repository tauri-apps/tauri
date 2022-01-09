// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::SnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};

use quote::{format_ident, quote, quote_spanned};
use syn::{
  parse::{Parse, ParseStream},
  spanned::Spanned,
  Data, DeriveInput, Error, Fields, Ident, ItemFn, LitStr, Token,
};

pub fn generate_run_fn(input: DeriveInput) -> TokenStream {
  let name = &input.ident;
  let data = &input.data;

  let mut is_async = false;
  let attrs = input.attrs;
  for attr in attrs {
    if attr.path.is_ident("cmd") {
      let _ = attr.parse_args_with(|input: ParseStream| {
        while let Some(token) = input.parse()? {
          if let TokenTree::Ident(ident) = token {
            is_async |= ident == "async";
          }
        }
        Ok(())
      });
    }
  }
  let maybe_await = if is_async { quote!(.await) } else { quote!() };
  let maybe_async = if is_async { quote!(async) } else { quote!() };

  let mut matcher;

  match data {
    Data::Enum(data_enum) => {
      matcher = TokenStream2::new();

      for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        let (fields_in_variant, variables) = match &variant.fields {
          Fields::Unit => (quote_spanned! { variant.span() => }, quote!()),
          Fields::Unnamed(fields) => {
            let mut variables = TokenStream2::new();
            for i in 0..fields.unnamed.len() {
              let variable_name = format_ident!("value{}", i);
              variables.extend(quote!(#variable_name,));
            }
            (quote_spanned! { variant.span() => (#variables) }, variables)
          }
          Fields::Named(fields) => {
            let mut variables = TokenStream2::new();
            for field in &fields.named {
              let ident = field.ident.as_ref().unwrap();
              variables.extend(quote!(#ident,));
            }
            (
              quote_spanned! { variant.span() => { #variables } },
              variables,
            )
          }
        };

        let mut variant_execute_function_name = format_ident!(
          "{}",
          variant_name.to_string().to_snake_case().to_lowercase()
        );
        variant_execute_function_name.set_span(variant_name.span());

        matcher.extend(quote_spanned! {
          variant.span() => #name::#variant_name #fields_in_variant => #name::#variant_execute_function_name(context, #variables)#maybe_await.map(Into::into),
        });
      }
    }
    _ => {
      return Error::new(
        Span::call_site(),
        "CommandModule is only implemented for enums",
      )
      .to_compile_error()
      .into()
    }
  };

  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let expanded = quote! {
      impl #impl_generics #name #ty_generics #where_clause {
        pub #maybe_async fn run<R: crate::Runtime>(self, context: crate::endpoints::InvokeContext<R>) -> crate::Result<crate::endpoints::InvokeResponse> {
          match self {
            #matcher
          }
        }
      }
  };

  TokenStream::from(expanded)
}

/// Attributes for the module enum variant handler.
pub struct HandlerAttributes {
  allowlist: Ident,
  error_message: String,
}

impl Parse for HandlerAttributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let allowlist = input.parse()?;
    input.parse::<Token![,]>()?;
    let raw: LitStr = input.parse()?;
    let error_message = raw.value();
    Ok(Self {
      allowlist,
      error_message,
    })
  }
}

pub fn command_handler(attributes: HandlerAttributes, function: ItemFn) -> TokenStream2 {
  let allowlist = attributes.allowlist;
  let error_message = attributes.error_message.as_str();
  let signature = function.sig.clone();

  quote!(
    #[cfg(#allowlist)]
    #function

    #[cfg(not(#allowlist))]
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    #signature {
      Err(crate::Error::ApiNotAllowlisted(
        #error_message.to_string(),
      ))
    }
  )
}
