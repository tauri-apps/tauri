// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Useful items for custom commands.

use crate::hooks::InvokeError;
use crate::runtime::Runtime;
use crate::InvokeMessage;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

/// Represents a custom command.
pub struct CommandItem<'a, R: Runtime> {
  /// The name of the command, e.g. `handler` on `#[command] fn handler(value: u64)`
  pub name: &'static str,

  /// The key of the command item, e.g. `value` on `#[command] fn handler(value: u64)`
  pub key: &'static str,

  /// The [`InvokeMessage`] that was passed to this command.
  pub message: &'a InvokeMessage<R>,
}

/// Trait implemented by command arguments to derive a value from a [`CommandItem`].
///
/// # Command Arguments
///
/// A command argument is any type that represents an item parsable from a [`CommandItem`]. Most
/// implementations will use the data stored in [`InvokeMessage`] since [`CommandItem`] is mostly a
/// wrapper around it.
///
/// # Provided Implementations
///
/// Tauri implements [`CommandArg`] automatically for a number of types.
/// * [`crate::Window`]
/// * [`crate::State`]
/// * `T where T: serde::Deserialize`
///   * Any type that implements `Deserialize` can automatically be used as a [`CommandArg`].
pub trait CommandArg<'de, R: Runtime>: Sized {
  /// Derives an instance of `Self` from the [`CommandItem`].
  ///
  /// If the derivation fails, the corresponding message will be rejected using [`InvokeMessage#reject`].
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError>;
}

/// Automatically implement [`CommandArg`] for any type that can be deserialized.
impl<'de, D: Deserialize<'de>, R: Runtime> CommandArg<'de, R> for D {
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let arg = command.key;
    Self::deserialize(command).map_err(|e| crate::Error::InvalidArgs(arg, e).into())
  }
}

/// Pass the result of [`serde_json::Value::get`] into [`serde_json::Value`]'s deserializer.
///
/// Returns an error if the [`CommandItem`]'s key does not exist in the value.
macro_rules! pass {
  ($fn:ident, $($arg:ident: $argt:ty),+) => {
    fn $fn<V: Visitor<'de>>(self, $($arg: $argt),*) -> Result<V::Value, Self::Error> {
      use serde::de::Error;

      if self.key.is_empty() {
        return Err(serde_json::Error::custom(format!(
            "command {} has an argument with no name with a non-optional value",
            self.name
          )))
      }

      match self.message.payload.get(self.key) {
        Some(value) => value.$fn($($arg),*),
        None => {
          Err(serde_json::Error::custom(format!(
            "command {} missing required key {}",
            self.name, self.key
          )))
        }
      }
    }
  }
}

/// A [`Deserializer`] wrapper around [`CommandItem`].
///
/// If the key doesn't exist, an error will be returned if the deserialized type is not expecting
/// an optional item. If the key does exist, the value will be called with
/// [`Value`](serde_json::Value)'s [`Deserializer`] implementation.
impl<'de, R: Runtime> Deserializer<'de> for CommandItem<'de, R> {
  type Error = serde_json::Error;

  pass!(deserialize_any, visitor: V);
  pass!(deserialize_bool, visitor: V);
  pass!(deserialize_i8, visitor: V);
  pass!(deserialize_i16, visitor: V);
  pass!(deserialize_i32, visitor: V);
  pass!(deserialize_i64, visitor: V);
  pass!(deserialize_u8, visitor: V);
  pass!(deserialize_u16, visitor: V);
  pass!(deserialize_u32, visitor: V);
  pass!(deserialize_u64, visitor: V);
  pass!(deserialize_f32, visitor: V);
  pass!(deserialize_f64, visitor: V);
  pass!(deserialize_char, visitor: V);
  pass!(deserialize_str, visitor: V);
  pass!(deserialize_string, visitor: V);
  pass!(deserialize_bytes, visitor: V);
  pass!(deserialize_byte_buf, visitor: V);

  fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
    match self.message.payload.get(self.key) {
      Some(value) => value.deserialize_option(visitor),
      None => visitor.visit_none(),
    }
  }

  pass!(deserialize_unit, visitor: V);
  pass!(deserialize_unit_struct, name: &'static str, visitor: V);
  pass!(deserialize_newtype_struct, name: &'static str, visitor: V);
  pass!(deserialize_seq, visitor: V);
  pass!(deserialize_tuple, len: usize, visitor: V);

  pass!(
    deserialize_tuple_struct,
    name: &'static str,
    len: usize,
    visitor: V
  );

  pass!(deserialize_map, visitor: V);

  pass!(
    deserialize_struct,
    name: &'static str,
    fields: &'static [&'static str],
    visitor: V
  );

  pass!(
    deserialize_enum,
    name: &'static str,
    fields: &'static [&'static str],
    visitor: V
  );

  pass!(deserialize_identifier, visitor: V);
  pass!(deserialize_ignored_any, visitor: V);
}

/// [Autoref-based stable specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md)
///
/// Nothing in this module is considered stable.
#[doc(hidden)]
pub mod private {
  use crate::{runtime::Runtime, InvokeError, InvokeResolver};
  use futures::{FutureExt, TryFutureExt};
  use serde::Serialize;
  use serde_json::Value;
  use std::future::Future;

  // ===== impl Serialize =====

  pub struct SerializeTag;

  pub trait SerializeKind {
    #[inline(always)]
    fn blocking_kind(&self) -> SerializeTag {
      SerializeTag
    }

    #[inline(always)]
    fn async_kind(&self) -> SerializeTag {
      SerializeTag
    }
  }

  impl<T: Serialize> SerializeKind for &T {}

  impl SerializeTag {
    #[inline(always)]
    pub fn block<R, T>(self, value: T, resolver: InvokeResolver<R>)
    where
      R: Runtime,
      T: Serialize,
    {
      resolver.respond(Ok(value))
    }

    #[inline(always)]
    pub fn future<T>(self, value: T) -> impl Future<Output = Result<Value, InvokeError>>
    where
      T: Serialize,
    {
      std::future::ready(serde_json::to_value(value).map_err(InvokeError::from_serde_json))
    }
  }

  // ===== Result<impl Serialize, impl Into<InvokeError>> =====

  pub struct ResultTag;

  pub trait ResultKind {
    #[inline(always)]
    fn blocking_kind(&self) -> ResultTag {
      ResultTag
    }

    #[inline(always)]
    fn async_kind(&self) -> ResultTag {
      ResultTag
    }
  }

  impl<T: Serialize, E: Into<InvokeError>> ResultKind for Result<T, E> {}

  impl ResultTag {
    #[inline(always)]
    pub fn block<R, T, E>(self, value: Result<T, E>, resolver: InvokeResolver<R>)
    where
      R: Runtime,
      T: Serialize,
      E: Into<InvokeError>,
    {
      resolver.respond(value.map_err(Into::into))
    }

    #[inline(always)]
    pub fn future<T, E>(
      self,
      value: Result<T, E>,
    ) -> impl Future<Output = Result<Value, InvokeError>>
    where
      T: Serialize,
      E: Into<InvokeError>,
    {
      std::future::ready(
        value
          .map_err(Into::into)
          .and_then(|value| serde_json::to_value(value).map_err(InvokeError::from_serde_json)),
      )
    }
  }

  // ===== Future<Output = impl Serialize> =====

  pub struct FutureTag;

  pub trait FutureKind {
    #[inline(always)]
    fn async_kind(&self) -> FutureTag {
      FutureTag
    }
  }
  impl<T: Serialize, F: Future<Output = T>> FutureKind for &F {}

  impl FutureTag {
    #[inline(always)]
    pub fn future<T, F>(self, value: F) -> impl Future<Output = Result<Value, InvokeError>>
    where
      T: Serialize,
      F: Future<Output = T> + Send + 'static,
    {
      value.map(|value| serde_json::to_value(value).map_err(InvokeError::from_serde_json))
    }
  }

  // ===== Future<Output = Result<impl Serialize, impl Into<InvokeError>>> =====

  pub struct ResultFutureTag;

  pub trait ResultFutureKind {
    #[inline(always)]
    fn async_kind(&self) -> ResultFutureTag {
      ResultFutureTag
    }
  }

  impl<T: Serialize, E: Into<InvokeError>, F: Future<Output = Result<T, E>>> ResultFutureKind for F {}

  impl ResultFutureTag {
    #[inline(always)]
    pub fn future<T, E, F>(self, value: F) -> impl Future<Output = Result<Value, InvokeError>>
    where
      T: Serialize,
      E: Into<InvokeError>,
      F: Future<Output = Result<T, E>> + Send,
    {
      value.err_into().map(|result| {
        result.and_then(|value| serde_json::to_value(value).map_err(InvokeError::from_serde_json))
      })
    }
  }
}
