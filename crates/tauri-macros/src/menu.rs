// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  Expr, Token,
};

pub struct DoMenuItemInput {
  resources_table: Ident,
  rid: Ident,
  kind: Ident,
  var: Ident,
  expr: Expr,
  kinds: Vec<NegatedIdent>,
}

#[derive(Clone)]
struct NegatedIdent {
  negated: bool,
  ident: Ident,
}

impl NegatedIdent {
  fn new(ident: &str) -> Self {
    Self {
      negated: false,
      ident: Ident::new(ident, Span::call_site()),
    }
  }

  fn is_negated(&self) -> bool {
    self.negated
  }
}

impl Parse for NegatedIdent {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let negated_token = input.parse::<Token![!]>();
    let ident: Ident = input.parse()?;
    Ok(NegatedIdent {
      negated: negated_token.is_ok(),
      ident,
    })
  }
}

impl Parse for DoMenuItemInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let resources_table: Ident = input.parse()?;
    let _: Token![,] = input.parse()?;
    let rid: Ident = input.parse()?;
    let _: Token![,] = input.parse()?;
    let kind: Ident = input.parse()?;
    let _: Token![,] = input.parse()?;
    let _: Token![|] = input.parse()?;
    let var: Ident = input.parse()?;
    let _: Token![|] = input.parse()?;
    let expr: Expr = input.parse()?;
    let _: syn::Result<Token![,]> = input.parse();
    let kinds = Punctuated::<NegatedIdent, Token![|]>::parse_terminated(input)?;

    Ok(Self {
      resources_table,
      rid,
      kind,
      var,
      expr,
      kinds: kinds.into_iter().collect(),
    })
  }
}

pub fn do_menu_item(input: DoMenuItemInput) -> TokenStream {
  let DoMenuItemInput {
    rid,
    resources_table,
    kind,
    expr,
    var,
    mut kinds,
  } = input;

  let defaults = vec![
    NegatedIdent::new("Submenu"),
    NegatedIdent::new("MenuItem"),
    NegatedIdent::new("Predefined"),
    NegatedIdent::new("Check"),
    NegatedIdent::new("Icon"),
  ];

  if kinds.is_empty() {
    kinds.extend(defaults.clone());
  }

  let has_negated = kinds.iter().any(|n| n.is_negated());
  if has_negated {
    kinds.extend(defaults);
    kinds.sort_by(|a, b| a.ident.cmp(&b.ident));
    kinds.dedup_by(|a, b| a.ident == b.ident);
  }

  let (kinds, types): (Vec<Ident>, Vec<Ident>) = kinds
    .into_iter()
    .filter_map(|nident| {
      if nident.is_negated() {
        None
      } else {
        match nident.ident {
          i if i == "MenuItem" => Some((i, Ident::new("MenuItem", Span::call_site()))),
          i if i == "Submenu" => Some((i, Ident::new("Submenu", Span::call_site()))),
          i if i == "Predefined" => Some((i, Ident::new("PredefinedMenuItem", Span::call_site()))),
          i if i == "Check" => Some((i, Ident::new("CheckMenuItem", Span::call_site()))),
          i if i == "Icon" => Some((i, Ident::new("IconMenuItem", Span::call_site()))),
          _ => None,
        }
      }
    })
    .unzip();

  quote! {
    match #kind {
      #(
        ItemKind::#kinds => {
        let #var = #resources_table.get::<#types<R>>(#rid)?;
        #expr
      }
      )*
      _ => unreachable!(),
    }
  }
}
