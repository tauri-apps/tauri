// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri custom commands types and traits.
//!
//! You usually don't need to create these items yourself. These are created from [command](../attr.command.html)
//! attribute macro along the way and used by [`crate::generate_handler`] macro.

use crate::{
  ipc::{InvokeBody, InvokeError, InvokeMessage},
  Runtime,
};
use serde::{
  de::{Error, Visitor},
  Deserialize, Deserializer,
};

use tauri_utils::acl::resolved::ResolvedCommand;

/// Represents a custom command.
pub struct CommandItem<'a, R: Runtime> {
  /// Name of the plugin if this command targets one.
  pub plugin: Option<&'static str>,

  /// The name of the command, e.g. `handler` on `#[command] fn handler(value: u64)`
  pub name: &'static str,

  /// The key of the command item, e.g. `value` on `#[command] fn handler(value: u64)`
  pub key: &'static str,

  /// The [`InvokeMessage`] that was passed to this command.
  pub message: &'a InvokeMessage<R>,

  /// The resolved ACL for this command.
  pub acl: &'a Option<Vec<ResolvedCommand>>,
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
  fn from_command(command: CommandItem<'de, R>) -> Result<D, InvokeError> {
    let name = command.name;
    let arg = command.key;
    #[cfg(feature = "tracing")]
    let _span = tracing::trace_span!("ipc::request::deserialize_arg", arg = arg).entered();
    Self::deserialize(command).map_err(|e| crate::Error::InvalidArgs(name, arg, e).into())
  }
}

/// Pass the result of [`serde_json::Value::get`] into [`serde_json::Value`]'s deserializer.
///
/// Returns an error if the [`CommandItem`]'s key does not exist in the value.
macro_rules! pass {
  ($fn:ident, $($arg:ident: $argt:ty),+) => {
    fn $fn<V: Visitor<'de>>(self, $($arg: $argt),*) -> Result<V::Value, Self::Error> {
      if self.key.is_empty() {
        return Err(serde_json::Error::custom(format!(
            "command {} has an argument with no name with a non-optional value",
            self.name
          )))
      }

      match &self.message.payload {
        InvokeBody::Raw(_body) => {
          Err(serde_json::Error::custom(format!(
            "command {} expected a value for key {} but the IPC call used a bytes payload",
            self.name, self.key
          )))
        }
        InvokeBody::Json(v) => {
          match v.get(self.key) {
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
    match &self.message.payload {
      InvokeBody::Raw(_body) => Err(serde_json::Error::custom(format!(
        "command {} expected a value for key {} but the IPC call used a bytes payload",
        self.name, self.key
      ))),
      InvokeBody::Json(v) => match v.get(self.key) {
        Some(value) => value.deserialize_option(visitor),
        None => visitor.visit_none(),
      },
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
  use crate::{
    ipc::{InvokeError, InvokeResolver, InvokeResponse, InvokeResponseBody, IpcResponse},
    Runtime,
  };
  use futures_util::{FutureExt, TryFutureExt};
  use std::{future::Future, pin::Pin};
  #[cfg(feature = "tracing")]
  pub use tracing;

  // ===== impl IpcResponse =====

  pub struct ResponseTag(InvokeResponse);

  pub trait ResponseKind {
    fn blocking_kind(self) -> ResponseTag;

    fn async_kind(self) -> ResponseTag;
  }

  impl ResponseKind for Vec<u8> {
    #[inline(always)]
    fn blocking_kind(self) -> ResponseTag {
      ResponseTag(InvokeResponse::Ok(self.into()))
    }

    #[inline(always)]
    fn async_kind(self) -> ResponseTag {
      ResponseTag(InvokeResponse::Ok(self.into()))
    }
  }

  impl<T: IpcResponse + Clone> ResponseKind for &T {
    #[inline(always)]
    fn blocking_kind(self) -> ResponseTag {
      ResponseTag(self.clone().body().map_err(InvokeError::from_error).into())
    }

    #[inline(always)]
    fn async_kind(self) -> ResponseTag {
      ResponseTag(self.clone().body().map_err(InvokeError::from_error).into())
    }
  }

  impl ResponseTag {
    #[inline(always)]
    pub fn block<R>(self, resolver: InvokeResolver<R>)
    where
      R: Runtime,
    {
      resolver.respond(self.0)
    }

    #[inline(always)]
    pub fn future<T>(self) -> impl Future<Output = Result<InvokeResponseBody, InvokeError>>
    where
      T: IpcResponse,
    {
      std::future::ready(match self.0 {
        InvokeResponse::Ok(b) => Ok(b),
        InvokeResponse::Err(e) => Err(e),
      })
    }
  }

  // ===== Result<impl IpcResponse, impl Into<InvokeError>> =====

  pub struct ResultTag(InvokeResponse);

  pub trait ResultKind {
    fn blocking_kind(self) -> ResultTag;

    fn async_kind(self) -> ResultTag;
  }

  impl<T: IpcResponse, E: Into<InvokeError>> ResultKind for Result<T, E> {
    #[inline(always)]
    fn blocking_kind(self) -> ResultTag {
      ResultTag(
        self
          .map_err(Into::into)
          .and_then(|r| r.body().map_err(InvokeError::from_error))
          .into(),
      )
    }

    #[inline(always)]
    fn async_kind(self) -> ResultTag {
      ResultTag(
        self
          .map_err(Into::into)
          .and_then(|r| r.body().map_err(InvokeError::from_error))
          .into(),
      )
    }
  }

  impl ResultTag {
    #[inline(always)]
    pub fn block<R: Runtime>(self, resolver: InvokeResolver<R>) {
      resolver.respond(self.0)
    }

    #[inline(always)]
    pub fn future(self) -> impl Future<Output = Result<InvokeResponseBody, InvokeError>> {
      std::future::ready(match self.0 {
        InvokeResponse::Ok(b) => Ok(b),
        InvokeResponse::Err(e) => Err(e),
      })
    }
  }

  // ===== Future<Output = Vec<u8>> =====

  pub struct BufferFutureTag<F: Future<Output = Vec<u8>>>(Pin<Box<F>>);

  pub trait BufferFutureKind<F: Future<Output = Vec<u8>>> {
    fn async_kind(self) -> BufferFutureTag<F>;
  }

  impl<F: Future<Output = Vec<u8>>> BufferFutureKind<F> for F {
    #[inline(always)]
    fn async_kind(self) -> BufferFutureTag<F> {
      BufferFutureTag(Box::pin(self))
    }
  }

  impl<F: Future<Output = Vec<u8>>> BufferFutureTag<F> {
    #[inline(always)]
    pub fn future(self) -> impl Future<Output = Result<InvokeResponseBody, InvokeError>> {
      self.0.map(|value| Ok(InvokeResponseBody::Raw(value)))
    }
  }

  // ===== Future<Output = impl IpcResponse> =====

  pub struct FutureTag<T: IpcResponse, F: Future<Output = T>>(Pin<Box<F>>);

  pub trait FutureKind<T: IpcResponse, F: Future<Output = T>> {
    fn async_kind(self) -> FutureTag<T, F>;
  }

  impl<T: IpcResponse, F: Future<Output = T>> FutureKind<T, F> for F {
    #[inline(always)]
    fn async_kind(self) -> FutureTag<T, F> {
      FutureTag(Box::pin(self))
    }
  }

  impl<T: IpcResponse, F: Future<Output = T>> FutureTag<T, F> {
    #[inline(always)]
    pub fn future(self) -> impl Future<Output = Result<InvokeResponseBody, InvokeError>> {
      self
        .0
        .map(|value| value.body().map_err(InvokeError::from_error))
    }
  }

  // ===== Future<Output = Result<impl Serialize, impl Into<InvokeError>>> =====

  pub struct ResultFutureTag<T: IpcResponse, E: Into<InvokeError>, F: Future<Output = Result<T, E>>>(
    Pin<Box<F>>,
  );

  pub trait ResultFutureKind<T: IpcResponse, E: Into<InvokeError>, F: Future<Output = Result<T, E>>>
  {
    fn async_kind(self) -> ResultFutureTag<T, E, F>;
  }

  impl<T: IpcResponse, E: Into<InvokeError>, F: Future<Output = Result<T, E>>>
    ResultFutureKind<T, E, F> for F
  {
    #[inline(always)]
    fn async_kind(self) -> ResultFutureTag<T, E, F> {
      ResultFutureTag(Box::pin(self))
    }
  }

  impl<T: IpcResponse, E: Into<InvokeError>, F: Future<Output = Result<T, E>>>
    ResultFutureTag<T, E, F>
  {
    #[inline(always)]
    pub fn future(self) -> impl Future<Output = Result<InvokeResponseBody, InvokeError>> {
      self
        .0
        .err_into()
        .map(|result| result.and_then(|value| value.body().map_err(InvokeError::from_error)))
    }
  }
}
