// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::path::PathBuf;
use syn::{
  parse::{Parse, ParseBuffer},
  punctuated::Punctuated,
  Expr, ExprLit, Lit, LitBool, LitStr, Meta, PathArguments, PathSegment, Token,
};
use tauri_codegen::{context_codegen, get_config, ContextData};
use tauri_utils::{config::parse::does_supported_file_name_exist, platform::Target};

pub(crate) struct ContextItems {
  config_file: PathBuf,
  root: syn::Path,
  capabilities: Option<Vec<PathBuf>>,
  assets: Option<Expr>,
  test: bool,
}

impl Parse for ContextItems {
  fn parse(input: &ParseBuffer<'_>) -> syn::parse::Result<Self> {
    let target = std::env::var("TARGET")
      .or_else(|_| std::env::var("TAURI_ENV_TARGET_TRIPLE"))
      .as_deref()
      .map(Target::from_triple)
      .unwrap_or_else(|_| Target::current());

    let mut root = None;
    let mut capabilities = None;
    let mut assets = None;
    let mut test = false;
    let config_file = input.parse::<LitStr>().ok().map(|raw| {
      let _ = input.parse::<Token![,]>();
      let path = PathBuf::from(raw.value());
      if path.is_relative() {
        std::env::var("CARGO_MANIFEST_DIR")
          .map(|m| PathBuf::from(m).join(path))
          .map_err(|e| e.to_string())
      } else {
        Ok(path)
      }
      .and_then(|path| {
        if does_supported_file_name_exist(target, &path) {
          Ok(path)
        } else {
          Err(format!(
            "no file at path {} exists, expected tauri config file",
            path.display()
          ))
        }
      })
    });

    while let Ok(meta) = input.parse::<Meta>() {
      match meta {
        Meta::Path(p) => {
          root.replace(p);
        }
        Meta::NameValue(v) => {
          let ident = v.path.require_ident()?;
          match ident.to_string().as_str() {
            "capabilities" => {
              if let Expr::Array(array) = v.value {
                capabilities.replace(
                  array
                    .elems
                    .into_iter()
                    .map(|e| {
                      if let Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(s),
                      }) = e
                      {
                        Ok(s.value().into())
                      } else {
                        Err(syn::Error::new(
                          input.span(),
                          "unexpected expression for capability",
                        ))
                      }
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?,
                );
              } else {
                return Err(syn::Error::new(
                  input.span(),
                  "unexpected value for capabilities",
                ));
              }
            }
            "assets" => {
              assets.replace(v.value);
            }
            "test" => {
              if let Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, .. }),
                ..
              }) = v.value
              {
                test = value;
              } else {
                return Err(syn::Error::new(input.span(), "unexpected value for test"));
              }
            }
            name => {
              return Err(syn::Error::new(
                input.span(),
                format!("unknown attribute {name}"),
              ));
            }
          }
        }
        Meta::List(_) => {
          return Err(syn::Error::new(input.span(), "unexpected list input"));
        }
      }

      let _ = input.parse::<Token![,]>();
    }

    Ok(Self {
      config_file: config_file
        .unwrap_or_else(|| {
          std::env::var("CARGO_MANIFEST_DIR")
            .map(|m| PathBuf::from(m).join("tauri.conf.json"))
            .map_err(|e| e.to_string())
        })
        .map_err(|e| input.error(e))?,
      root: root.unwrap_or_else(|| {
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
          ident: Ident::new("tauri", Span::call_site()),
          arguments: PathArguments::None,
        });
        syn::Path {
          leading_colon: Some(Token![::](Span::call_site())),
          segments,
        }
      }),
      capabilities,
      assets,
      test,
    })
  }
}

pub(crate) fn generate_context(context: ContextItems) -> TokenStream {
  let context = get_config(&context.config_file)
    .map_err(|e| e.to_string())
    .map(|(config, config_parent)| ContextData {
      dev: cfg!(not(feature = "custom-protocol")),
      config,
      config_parent,
      root: context.root.to_token_stream(),
      capabilities: context.capabilities,
      assets: context.assets,
      test: context.test,
    })
    .and_then(|data| context_codegen(data).map_err(|e| e.to_string()));

  match context {
    Ok(code) => code,
    Err(error) => quote!(compile_error!(#error)),
  }
}
