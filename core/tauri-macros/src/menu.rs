// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
struct NegatedIdent(bool, Ident);

impl Parse for NegatedIdent {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let t = input.parse::<Token![!]>();
    let i: Ident = input.parse()?;
    Ok(NegatedIdent(t.is_ok(), i))
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
    NegatedIdent(false, Ident::new("Submenu", Span::call_site())),
    NegatedIdent(false, Ident::new("MenuItem", Span::call_site())),
    NegatedIdent(false, Ident::new("Predefined", Span::call_site())),
    NegatedIdent(false, Ident::new("Check", Span::call_site())),
    NegatedIdent(false, Ident::new("Icon", Span::call_site())),
  ];

  if kinds.is_empty() {
    kinds.extend(defaults.clone());
  }

  let has_negated = kinds.iter().any(|n| n.0);

  if has_negated {
    kinds.extend(defaults);
    kinds.sort_by(|a, b| a.1.cmp(&b.1));
    kinds.dedup_by(|a, b| a.1 == b.1);
  }

  let (kinds, types): (Vec<Ident>, Vec<Ident>) = kinds
    .into_iter()
    .filter_map(|nident| {
      if nident.0 {
        None
      } else {
        match nident.1 {
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
