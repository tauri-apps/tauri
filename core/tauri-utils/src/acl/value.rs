// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A [`Value`] that is used instead of [`toml::Value`] or [`serde_json::Value`]
//! to support both formats.

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A valid ACL number.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum Number {
  /// Represents an [`i64`].
  Int(i64),

  /// Represents a [`f64`].
  Float(f64),
}

impl From<i64> for Number {
  #[inline(always)]
  fn from(value: i64) -> Self {
    Self::Int(value)
  }
}

impl From<f64> for Number {
  #[inline(always)]
  fn from(value: f64) -> Self {
    Self::Float(value)
  }
}

/// All supported ACL values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum Value {
  /// Represents a [`bool`].
  Bool(bool),

  /// Represents a valid ACL [`Number`].
  Number(Number),

  /// Represents a [`String`].
  String(String),

  /// Represents a list of other [`Value`]s.
  List(Vec<Value>),

  /// Represents a map of [`String`] keys to [`Value`]s.
  Map(BTreeMap<String, Value>),
}

impl Value {
  /// TODO: implement [`serde::Deserializer`] directly to avoid serializing then deserializing
  pub fn deserialize<T: DeserializeOwned + Debug>(&self) -> Option<T> {
    dbg!(serde_json::to_string(self))
      .ok()
      .and_then(|s| dbg!(serde_json::from_str(&s).ok()))
  }
}

impl From<bool> for Value {
  #[inline(always)]
  fn from(value: bool) -> Self {
    Self::Bool(value)
  }
}

impl<T: Into<Number>> From<T> for Value {
  #[inline(always)]
  fn from(value: T) -> Self {
    Self::Number(value.into())
  }
}

impl From<String> for Value {
  #[inline(always)]
  fn from(value: String) -> Self {
    Value::String(value)
  }
}

impl From<toml::Value> for Value {
  #[inline(always)]
  fn from(value: toml::Value) -> Self {
    use toml::Value as Toml;

    match value {
      Toml::String(s) => s.into(),
      Toml::Integer(i) => i.into(),
      Toml::Float(f) => f.into(),
      Toml::Boolean(b) => b.into(),
      Toml::Datetime(d) => d.to_string().into(),
      Toml::Array(a) => Value::List(a.into_iter().map(Value::from).collect()),
      Toml::Table(t) => Value::Map(t.into_iter().map(|(k, v)| (k, v.into())).collect()),
    }
  }
}

#[cfg(feature = "build")]
mod build {
  use std::convert::identity;

  use crate::tokens::*;

  use super::*;
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::Number };

      tokens.append_all(match self {
        Self::Int(i) => {
          quote! { #prefix::Int(#i) }
        }
        Self::Float(f) => {
          quote! { #prefix::Float (#f) }
        }
      });
    }
  }

  impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::Value };

      tokens.append_all(match self {
        Value::Bool(bool) => quote! { #prefix::Bool(#bool) },
        Value::Number(number) => quote! { #prefix::Number(#number) },
        Value::String(str) => {
          let s = str_lit(str);
          quote! { #prefix::String(#s) }
        }
        Value::List(vec) => {
          let items = vec_lit(vec, identity);
          quote! { #prefix::List(#items) }
        }
        Value::Map(map) => {
          let map = map_lit(
            quote! { ::std::collections::BTreeMap },
            map,
            str_lit,
            identity,
          );
          quote! { #prefix::Map(#map) }
        }
      });
    }
  }
}
