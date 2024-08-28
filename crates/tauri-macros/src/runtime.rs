// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
  parse_quote, DeriveInput, Error, GenericParam, Ident, ItemTrait, ItemType, Token, Type, TypeParam,
};

#[derive(Clone)]
pub(crate) enum Input {
  Derive(DeriveInput),
  Trait(ItemTrait),
  Type(ItemType),
}

impl Parse for Input {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    input
      .parse::<DeriveInput>()
      .map(Self::Derive)
      .or_else(|_| input.parse().map(Self::Trait))
      .or_else(|_| input.parse().map(Self::Type))
      .map_err(|_| {
        Error::new(
          input.span(),
          "default_runtime only supports `struct`, `enum`, `type`, or `trait` definitions",
        )
      })
  }
}

impl Input {
  fn last_param_mut(&mut self) -> Option<&mut GenericParam> {
    match self {
      Input::Derive(d) => d.generics.params.last_mut(),
      Input::Trait(t) => t.generics.params.last_mut(),
      Input::Type(t) => t.generics.params.last_mut(),
    }
  }
}

impl ToTokens for Input {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Input::Derive(d) => d.to_tokens(tokens),
      Input::Trait(t) => t.to_tokens(tokens),
      Input::Type(t) => t.to_tokens(tokens),
    }
  }
}

/// The default runtime type to enable when the provided feature is enabled.
pub(crate) struct Attributes {
  default_type: Type,
  feature: Ident,
}

impl Parse for Attributes {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let default_type = input.parse()?;
    input.parse::<Token![,]>()?;
    Ok(Attributes {
      default_type,
      feature: input.parse()?,
    })
  }
}

pub(crate) fn default_runtime(attributes: Attributes, input: Input) -> TokenStream {
  // create a new copy to manipulate for the wry feature flag
  let mut wry = input.clone();
  let wry_runtime = wry
    .last_param_mut()
    .expect("default_runtime requires the item to have at least 1 generic parameter");

  // set the default value of the last generic parameter to the provided runtime type
  match wry_runtime {
    GenericParam::Type(
      param @ TypeParam {
        eq_token: None,
        default: None,
        ..
      },
    ) => {
      param.eq_token = Some(parse_quote!(=));
      param.default = Some(attributes.default_type);
    }
    _ => {
      panic!("DefaultRuntime requires the last parameter to not have a default value")
    }
  };

  let feature = attributes.feature.to_string();

  quote!(
    #[cfg(feature = #feature)]
    #wry

    #[cfg(not(feature = #feature))]
    #input
  )
}
