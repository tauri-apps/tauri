// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Useful items for custom commands.

use crate::{InvokeMessage, Params};
use serde::de::Visitor;
use serde::Deserializer;

/// A [`Deserializer`] wrapper around [`Value::get`].
///
/// If the key doesn't exist, an error will be returned if the deserialized type is not expecting
/// an optional item. If the key does exist, the value will be called with [`Value`]'s
/// [`Deserializer`] implementation.
struct KeyedValue<'de> {
  command: &'de str,
  key: &'de str,
  value: &'de serde_json::Value,
}

macro_rules! kv_value {
  ($s:ident) => {{
    use serde::de::Error;

    match $s.value.get($s.key) {
      Some(value) => value,
      None => {
        return Err(serde_json::Error::custom(format!(
          "command {} missing required key {}",
          $s.command, $s.key
        )))
      }
    }
  }};
}

impl<'de> Deserializer<'de> for KeyedValue<'de> {
  type Error = serde_json::Error;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_any(visitor)
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_bool(visitor)
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_i8(visitor)
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_i16(visitor)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_i32(visitor)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_i64(visitor)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_u8(visitor)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_u16(visitor)
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_u32(visitor)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_u64(visitor)
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_f32(visitor)
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_f64(visitor)
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_char(visitor)
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_str(visitor)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_string(visitor)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_bytes(visitor)
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_byte_buf(visitor)
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value.get(self.key) {
      Some(value) => value.deserialize_option(visitor),
      None => visitor.visit_none(),
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_unit(visitor)
  }

  fn deserialize_unit_struct<V>(
    self,
    name: &'static str,
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_unit_struct(name, visitor)
  }

  fn deserialize_newtype_struct<V>(
    self,
    name: &'static str,
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_newtype_struct(name, visitor)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_seq(visitor)
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_tuple(len, visitor)
  }

  fn deserialize_tuple_struct<V>(
    self,
    name: &'static str,
    len: usize,
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_tuple_struct(name, len, visitor)
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_map(visitor)
  }

  fn deserialize_struct<V>(
    self,
    name: &'static str,
    fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_struct(name, fields, visitor)
  }

  fn deserialize_enum<V>(
    self,
    name: &'static str,
    variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_enum(name, variants, visitor)
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_identifier(visitor)
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    kv_value!(self).deserialize_ignored_any(visitor)
  }
}

/// Trait implemented by command arguments to derive a value from a [`InvokeMessage`].
/// [`tauri::Window`], [`tauri::State`] and types that implements [`Deserialize`] automatically implements this trait.
pub trait FromCommand<'de, P: Params>: Sized {
  /// Derives an instance of `Self` from the [`InvokeMessage`].
  /// If the derivation fails, the corresponding message will be rejected using [`InvokeMessage#reject`].
  ///
  /// # Arguments
  /// - `command`: the command value passed to invoke, e.g. `initialize` on `invoke('initialize', {})`.
  /// - `key`: The name of the variable in the command handler, e.g. `value` on `#[command] fn handler(value: u64)`
  /// - `message`: The [`InvokeMessage`] instance.
  fn from_command(
    command: &'de str,
    key: &'de str,
    message: &'de InvokeMessage<P>,
  ) -> ::core::result::Result<Self, serde_json::Error>;
}

impl<'de, D: serde::Deserialize<'de>, P: Params> FromCommand<'de, P> for D {
  fn from_command(
    command: &'de str,
    key: &'de str,
    message: &'de InvokeMessage<P>,
  ) -> ::core::result::Result<Self, serde_json::Error> {
    D::deserialize(KeyedValue {
      command,
      key,
      value: &message.payload.inner,
    })
  }
}
