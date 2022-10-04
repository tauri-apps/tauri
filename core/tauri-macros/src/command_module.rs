// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::{ToLowerCamelCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::{format_ident, quote, quote_spanned};
use syn::{
  parse::{Parse, ParseStream},
  parse_quote,
  spanned::Spanned,
  Data, DeriveInput, Error, Fields, Ident, ItemFn, LitStr, Token,
};

pub(crate) fn generate_command_enum(mut input: DeriveInput) -> TokenStream {
  let mut deserialize_functions = TokenStream2::new();
  let mut errors = TokenStream2::new();

  input.attrs.push(parse_quote!(#[allow(dead_code)]));

  match &mut input.data {
    Data::Enum(data_enum) => {
      for variant in &mut data_enum.variants {
        let mut feature: Option<Ident> = None;
        let mut error_message: Option<String> = None;

        for attr in &variant.attrs {
          if attr.path.is_ident("cmd") {
            let r = attr
              .parse_args_with(|input: ParseStream| {
                if let Ok(f) = input.parse::<Ident>() {
                  feature.replace(f);
                  input.parse::<Token![,]>()?;
                  let error_message_raw: LitStr = input.parse()?;
                  error_message.replace(error_message_raw.value());
                }
                Ok(quote!())
              })
              .unwrap_or_else(syn::Error::into_compile_error);
            errors.extend(r);
          }
        }

        if let Some(f) = feature {
          let error_message = if let Some(e) = error_message {
            let e = e.to_string();
            quote!(#e)
          } else {
            quote!("This API is not enabled in the allowlist.")
          };

          let deserialize_function_name = quote::format_ident!("__{}_deserializer", variant.ident);
          deserialize_functions.extend(quote! {
            #[cfg(not(#f))]
            #[allow(non_snake_case)]
            fn #deserialize_function_name<'de, D, T>(deserializer: D) -> ::std::result::Result<T, D::Error>
            where
              D: ::serde::de::Deserializer<'de>,
            {
              ::std::result::Result::Err(::serde::de::Error::custom(crate::Error::ApiNotAllowlisted(#error_message.into()).to_string()))
            }
          });

          let deserialize_function_name = deserialize_function_name.to_string();

          variant
          .attrs
          .push(parse_quote!(#[cfg_attr(not(#f), serde(deserialize_with = #deserialize_function_name))]));
        }
      }
    }
    _ => {
      return Error::new(
        Span::call_site(),
        "`command_enum` is only implemented for enums",
      )
      .to_compile_error()
      .into()
    }
  };

  TokenStream::from(quote! {
    #errors
    #input
    #deserialize_functions
  })
}

pub(crate) fn generate_run_fn(input: DeriveInput) -> TokenStream {
  let name = &input.ident;
  let data = &input.data;

  let mut errors = TokenStream2::new();

  let mut is_async = false;

  let attrs = input.attrs;
  for attr in attrs {
    if attr.path.is_ident("cmd") {
      let r = attr
        .parse_args_with(|input: ParseStream| {
          if let Ok(token) = input.parse::<Ident>() {
            is_async = token == "async";
          }
          Ok(quote!())
        })
        .unwrap_or_else(syn::Error::into_compile_error);
      errors.extend(r);
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

        let mut feature = None;

        for attr in &variant.attrs {
          if attr.path.is_ident("cmd") {
            let r = attr
              .parse_args_with(|input: ParseStream| {
                if let Ok(f) = input.parse::<Ident>() {
                  feature.replace(f);
                  input.parse::<Token![,]>()?;
                  let _: LitStr = input.parse()?;
                }
                Ok(quote!())
              })
              .unwrap_or_else(syn::Error::into_compile_error);
            errors.extend(r);
          }
        }

        let maybe_feature_check = if let Some(f) = feature {
          quote!(#[cfg(#f)])
        } else {
          quote!()
        };

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
          variant.span() => #maybe_feature_check #name::#variant_name #fields_in_variant => #name::#variant_execute_function_name(context, #variables)#maybe_await.map(Into::into),
        });
      }

      matcher.extend(quote! {
        _ => Err(crate::error::into_anyhow("API not in the allowlist (https://tauri.app/docs/api/config#tauri.allowlist)")),
      });
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
    #errors
    impl #impl_generics #name #ty_generics #where_clause {
        pub #maybe_async fn run<R: crate::Runtime>(self, context: crate::endpoints::InvokeContext<R>) -> super::Result<crate::endpoints::InvokeResponse> {
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
}

impl Parse for HandlerAttributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(Self {
      allowlist: input.parse()?,
    })
  }
}

pub enum AllowlistCheckKind {
  Runtime,
  Serde,
}

pub struct HandlerTestAttributes {
  allowlist: Ident,
  error_message: String,
  allowlist_check_kind: AllowlistCheckKind,
}

impl Parse for HandlerTestAttributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let allowlist = input.parse()?;
    input.parse::<Token![,]>()?;
    let error_message_raw: LitStr = input.parse()?;
    let error_message = error_message_raw.value();
    let allowlist_check_kind =
      if let (Ok(_), Ok(i)) = (input.parse::<Token![,]>(), input.parse::<Ident>()) {
        if i == "runtime" {
          AllowlistCheckKind::Runtime
        } else {
          AllowlistCheckKind::Serde
        }
      } else {
        AllowlistCheckKind::Serde
      };

    Ok(Self {
      allowlist,
      error_message,
      allowlist_check_kind,
    })
  }
}

pub fn command_handler(attributes: HandlerAttributes, function: ItemFn) -> TokenStream2 {
  let allowlist = attributes.allowlist;

  quote!(
    #[cfg(#allowlist)]
    #function
  )
}

pub fn command_test(attributes: HandlerTestAttributes, function: ItemFn) -> TokenStream2 {
  let allowlist = attributes.allowlist;
  let error_message = attributes.error_message.as_str();
  let signature = function.sig.clone();

  let enum_variant_name = function.sig.ident.to_string().to_lower_camel_case();
  let response = match attributes.allowlist_check_kind {
    AllowlistCheckKind::Runtime => {
      let test_name = function.sig.ident.clone();
      quote!(super::Cmd::#test_name(crate::test::mock_invoke_context()))
    }
    AllowlistCheckKind::Serde => quote! {
      serde_json::from_str::<super::Cmd>(&format!(r#"{{ "cmd": "{}", "data": null }}"#, #enum_variant_name))
    },
  };

  quote!(
    #[cfg(#allowlist)]
    #function

    #[cfg(not(#allowlist))]
    #[allow(unused_variables)]
    #[quickcheck_macros::quickcheck]
    #signature {
      if let Err(e) = #response {
        assert!(e.to_string().contains(#error_message));
      } else {
        panic!("unexpected response");
      }
    }
  )
}
