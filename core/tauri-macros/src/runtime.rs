// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, DeriveInput, GenericParam, Ident, Token, Type, TypeParam};

/// The default runtime type to enable when the provided feature is enabled.
pub(crate) struct Attributes {
  default_type: Type,
  feature: Ident,
}

impl Parse for Attributes {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let default_type = input.parse()?;
    input.parse::<Token![,]>()?;
    Ok(Attributes {
      default_type,
      feature: input.parse()?,
    })
  }
}

pub(crate) fn default_runtime(attributes: Attributes, input: DeriveInput) -> TokenStream {
  // create a new copy to manipulate for the wry feature flag
  let mut wry = input.clone();
  let wry_runtime = wry
    .generics
    .params
    .last_mut()
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
