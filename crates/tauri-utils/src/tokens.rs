// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Utilities to implement [`ToTokens`] for a type.

use std::path::Path;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde_json::Value as JsonValue;
use url::Url;

/// Write a `TokenStream` of the `$struct`'s fields to the `$tokens`.
///
/// All fields must represent a binding of the same name that implements `ToTokens`.
#[macro_export]
macro_rules! literal_struct {
  ($tokens:ident, $struct:path, $($field:ident),+) => {
    $tokens.append_all(quote! {
      $struct {
        $($field: #$field),+
      }
    })
  };
}

/// Create a `String` constructor `TokenStream`.
///
/// e.g. `"Hello World" -> String::from("Hello World").
/// This takes a `&String` to reduce casting all the `&String` -> `&str` manually.
pub fn str_lit(s: impl AsRef<str>) -> TokenStream {
  let s = s.as_ref();
  quote! { #s.into() }
}

/// Create an `Option` constructor `TokenStream`.
pub fn opt_lit(item: Option<&impl ToTokens>) -> TokenStream {
  match item {
    None => quote! { ::core::option::Option::None },
    Some(item) => quote! { ::core::option::Option::Some(#item) },
  }
}

/// Create an `Option` constructor `TokenStream` over an owned [`ToTokens`] impl type.
pub fn opt_lit_owned(item: Option<impl ToTokens>) -> TokenStream {
  match item {
    None => quote! { ::core::option::Option::None },
    Some(item) => quote! { ::core::option::Option::Some(#item) },
  }
}

/// Helper function to combine an `opt_lit` with `str_lit`.
pub fn opt_str_lit(item: Option<impl AsRef<str>>) -> TokenStream {
  opt_lit(item.map(str_lit).as_ref())
}

/// Helper function to combine an `opt_lit` with a list of `str_lit`
pub fn opt_vec_lit<Raw, Tokens>(
  item: Option<impl IntoIterator<Item = Raw>>,
  map: impl Fn(Raw) -> Tokens,
) -> TokenStream
where
  Tokens: ToTokens,
{
  opt_lit(item.map(|list| vec_lit(list, map)).as_ref())
}

/// Create a `Vec` constructor, mapping items with a function that spits out `TokenStream`s.
pub fn vec_lit<Raw, Tokens>(
  list: impl IntoIterator<Item = Raw>,
  map: impl Fn(Raw) -> Tokens,
) -> TokenStream
where
  Tokens: ToTokens,
{
  let items = list.into_iter().map(map);
  quote! { vec![#(#items),*] }
}

/// Create a `PathBuf` constructor `TokenStream`.
///
/// e.g. `"Hello World" -> String::from("Hello World").
pub fn path_buf_lit(s: impl AsRef<Path>) -> TokenStream {
  let s = s.as_ref().to_string_lossy().into_owned();
  quote! { ::std::path::PathBuf::from(#s) }
}

/// Creates a `Url` constructor `TokenStream`.
pub fn url_lit(url: &Url) -> TokenStream {
  let url = url.as_str();
  quote! { #url.parse().unwrap() }
}

/// Create a map constructor, mapping keys and values with other `TokenStream`s.
///
/// This function is pretty generic because the types of keys AND values get transformed.
pub fn map_lit<Map, Key, Value, TokenStreamKey, TokenStreamValue, FuncKey, FuncValue>(
  map_type: TokenStream,
  map: Map,
  map_key: FuncKey,
  map_value: FuncValue,
) -> TokenStream
where
  <Map as IntoIterator>::IntoIter: ExactSizeIterator,
  Map: IntoIterator<Item = (Key, Value)>,
  TokenStreamKey: ToTokens,
  TokenStreamValue: ToTokens,
  FuncKey: Fn(Key) -> TokenStreamKey,
  FuncValue: Fn(Value) -> TokenStreamValue,
{
  let ident = quote::format_ident!("map");
  let map = map.into_iter();

  if map.len() > 0 {
    let items = map.map(|(key, value)| {
      let key = map_key(key);
      let value = map_value(value);
      quote! { #ident.insert(#key, #value); }
    });

    quote! {{
      let mut #ident = #map_type::new();
      #(#items)*
      #ident
    }}
  } else {
    quote! { #map_type::new() }
  }
}

/// Create a `serde_json::Value` variant `TokenStream` for a number
pub fn json_value_number_lit(num: &serde_json::Number) -> TokenStream {
  // See <https://docs.rs/serde_json/1/serde_json/struct.Number.html> for guarantees
  let prefix = quote! { ::serde_json::Value };
  if num.is_u64() {
    // guaranteed u64
    let num = num.as_u64().unwrap();
    quote! { #prefix::Number(#num.into()) }
  } else if num.is_i64() {
    // guaranteed i64
    let num = num.as_i64().unwrap();
    quote! { #prefix::Number(#num.into()) }
  } else if num.is_f64() {
    // guaranteed f64
    let num = num.as_f64().unwrap();
    quote! { #prefix::Number(::serde_json::Number::from_f64(#num).unwrap(/* safe to unwrap, guaranteed f64 */)) }
  } else {
    // invalid number
    quote! { #prefix::Null }
  }
}

/// Create a `serde_json::Value` constructor `TokenStream`
pub fn json_value_lit(jv: &JsonValue) -> TokenStream {
  let prefix = quote! { ::serde_json::Value };

  match jv {
    JsonValue::Null => quote! { #prefix::Null },
    JsonValue::Bool(bool) => quote! { #prefix::Bool(#bool) },
    JsonValue::Number(number) => json_value_number_lit(number),
    JsonValue::String(str) => {
      let s = str_lit(str);
      quote! { #prefix::String(#s) }
    }
    JsonValue::Array(vec) => {
      let items = vec.iter().map(json_value_lit);
      quote! { #prefix::Array(vec![#(#items),*]) }
    }
    JsonValue::Object(map) => {
      let map = map_lit(quote! { ::serde_json::Map }, map, str_lit, json_value_lit);
      quote! { #prefix::Object(#map) }
    }
  }
}
