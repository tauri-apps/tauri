// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A [`Value`] that is used instead of [`toml::Value`] or [`serde_json::Value`]
//! to support both formats.

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// A valid ACL number.
#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, PartialOrd)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, PartialOrd)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Value {
  /// Represents a null JSON value.
  Null,

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

impl From<Value> for serde_json::Value {
  fn from(value: Value) -> Self {
    match value {
      Value::Null => serde_json::Value::Null,
      Value::Bool(b) => serde_json::Value::Bool(b),
      Value::Number(Number::Float(f)) => {
        serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
      }
      Value::Number(Number::Int(i)) => serde_json::Value::Number(i.into()),
      Value::String(s) => serde_json::Value::String(s),
      Value::List(list) => serde_json::Value::Array(list.into_iter().map(Into::into).collect()),
      Value::Map(map) => serde_json::Value::Object(
        map
          .into_iter()
          .map(|(key, value)| (key, value.into()))
          .collect(),
      ),
    }
  }
}

impl From<serde_json::Value> for Value {
  fn from(value: serde_json::Value) -> Self {
    match value {
      serde_json::Value::Null => Value::Null,
      serde_json::Value::Bool(b) => Value::Bool(b),
      serde_json::Value::Number(n) => Value::Number(if let Some(f) = n.as_f64() {
        Number::Float(f)
      } else if let Some(n) = n.as_u64() {
        Number::Int(n as i64)
      } else if let Some(n) = n.as_i64() {
        Number::Int(n)
      } else {
        Number::Int(0)
      }),
      serde_json::Value::String(s) => Value::String(s),
      serde_json::Value::Array(list) => Value::List(list.into_iter().map(Into::into).collect()),
      serde_json::Value::Object(map) => Value::Map(
        map
          .into_iter()
          .map(|(key, value)| (key, value.into()))
          .collect(),
      ),
    }
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
        Value::Null => quote! { #prefix::Null },
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
