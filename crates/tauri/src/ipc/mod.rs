// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to Inter Procedure Call(IPC).
//!
//! This module includes utilities to send messages to the JS layer of the webview.

use std::sync::{Arc, Mutex};

use futures_util::Future;
use http::HeaderMap;
use serde::{
  de::{DeserializeOwned, IntoDeserializer},
  Deserialize, Serialize,
};
use serde_json::Value as JsonValue;
pub use serialize_to_javascript::Options as SerializeOptions;
use tauri_macros::default_runtime;
use tauri_utils::acl::resolved::ResolvedCommand;

use crate::{webview::Webview, Runtime, StateManager};

mod authority;
pub(crate) mod channel;
mod command;
pub(crate) mod format_callback;
pub(crate) mod protocol;

pub use authority::{
  CapabilityBuilder, CommandScope, GlobalScope, Origin, RuntimeAuthority, RuntimeCapability,
  ScopeObject, ScopeObjectMatch, ScopeValue,
};
pub use channel::{Channel, JavaScriptChannelId};
pub use command::{private, CommandArg, CommandItem};

/// A closure that is run every time Tauri receives a message it doesn't explicitly handle.
pub type InvokeHandler<R> = dyn Fn(Invoke<R>) -> bool + Send + Sync + 'static;

/// A closure that is responsible for respond a JS message.
pub type InvokeResponder<R> =
  dyn Fn(&Webview<R>, &str, &InvokeResponse, CallbackFn, CallbackFn) + Send + Sync + 'static;
/// Similar to [`InvokeResponder`] but taking owned arguments.
pub type OwnedInvokeResponder<R> =
  dyn FnOnce(Webview<R>, String, InvokeResponse, CallbackFn, CallbackFn) + Send + 'static;

/// Possible values of an IPC payload.
///
/// ### Android
/// On Android, [InvokeBody::Raw] is not supported. The enum will always contain [InvokeBody::Json].
/// When targeting Android Devices, consider passing raw bytes as a base64 [[std::string::String]], which is still more efficient than passing them as a number array in [InvokeBody::Json]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum InvokeBody {
  /// Json payload.
  Json(JsonValue),
  /// Bytes payload.
  Raw(Vec<u8>),
}

impl Default for InvokeBody {
  fn default() -> Self {
    Self::Json(Default::default())
  }
}

impl From<JsonValue> for InvokeBody {
  fn from(value: JsonValue) -> Self {
    Self::Json(value)
  }
}

impl From<Vec<u8>> for InvokeBody {
  fn from(value: Vec<u8>) -> Self {
    Self::Raw(value)
  }
}

impl InvokeBody {
  #[cfg(mobile)]
  pub(crate) fn into_json(self) -> JsonValue {
    match self {
      Self::Json(v) => v,
      Self::Raw(v) => {
        JsonValue::Array(v.into_iter().map(|n| JsonValue::Number(n.into())).collect())
      }
    }
  }
}

/// Possible values of an IPC response.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum InvokeResponseBody {
  /// Json payload.
  Json(String),
  /// Bytes payload.
  Raw(Vec<u8>),
}

impl From<String> for InvokeResponseBody {
  fn from(value: String) -> Self {
    Self::Json(value)
  }
}

impl From<Vec<u8>> for InvokeResponseBody {
  fn from(value: Vec<u8>) -> Self {
    Self::Raw(value)
  }
}

impl From<InvokeBody> for InvokeResponseBody {
  fn from(value: InvokeBody) -> Self {
    match value {
      InvokeBody::Json(v) => Self::Json(serde_json::to_string(&v).unwrap()),
      InvokeBody::Raw(v) => Self::Raw(v),
    }
  }
}

impl IpcResponse for InvokeResponseBody {
  fn body(self) -> crate::Result<InvokeResponseBody> {
    Ok(self)
  }
}

impl InvokeResponseBody {
  /// Attempts to deserialize the response.
  pub fn deserialize<T: DeserializeOwned>(self) -> serde_json::Result<T> {
    match self {
      Self::Json(v) => serde_json::from_str(&v),
      Self::Raw(v) => T::deserialize(v.into_deserializer()),
    }
  }
}

/// The IPC request.
///
/// Includes the `body` and `headers` parameters of a Tauri command invocation.
/// This allows commands to accept raw bytes - on all platforms except Android.
#[derive(Debug)]
pub struct Request<'a> {
  body: &'a InvokeBody,
  headers: &'a HeaderMap,
}

impl<'a> Request<'a> {
  /// The request body.
  pub fn body(&self) -> &InvokeBody {
    self.body
  }

  /// Thr request headers.
  pub fn headers(&self) -> &HeaderMap {
    self.headers
  }
}

impl<'a, R: Runtime> CommandArg<'a, R> for Request<'a> {
  /// Returns the invoke [`Request`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    Ok(Self {
      body: command.message.payload(),
      headers: command.message.headers(),
    })
  }
}

/// Marks a type as a response to an IPC call.
pub trait IpcResponse {
  /// Resolve the IPC response body.
  fn body(self) -> crate::Result<InvokeResponseBody>;
}

impl<T: Serialize> IpcResponse for T {
  fn body(self) -> crate::Result<InvokeResponseBody> {
    serde_json::to_string(&self)
      .map(Into::into)
      .map_err(Into::into)
  }
}

/// The IPC response.
pub struct Response {
  body: InvokeResponseBody,
}

impl IpcResponse for Response {
  fn body(self) -> crate::Result<InvokeResponseBody> {
    Ok(self.body)
  }
}

impl Response {
  /// Defines a response with the given body.
  pub fn new(body: impl Into<InvokeResponseBody>) -> Self {
    Self { body: body.into() }
  }
}

/// The message and resolver given to a custom command.
///
/// This struct is used internally by macros and is explicitly **NOT** stable.
#[default_runtime(crate::Wry, wry)]
pub struct Invoke<R: Runtime> {
  /// The message passed.
  pub message: InvokeMessage<R>,

  /// The resolver of the message.
  pub resolver: InvokeResolver<R>,

  /// Resolved ACL for this IPC invoke.
  pub acl: Option<Vec<ResolvedCommand>>,
}

/// Error response from an [`InvokeMessage`].
#[derive(Debug)]
pub struct InvokeError(pub serde_json::Value);

impl InvokeError {
  /// Create an [`InvokeError`] as a string of the [`std::error::Error`] message.
  #[inline(always)]
  pub fn from_error<E: std::error::Error>(error: E) -> Self {
    Self(serde_json::Value::String(error.to_string()))
  }

  /// Create an [`InvokeError`] as a string of the [`anyhow::Error`] message.
  #[inline(always)]
  pub fn from_anyhow(error: anyhow::Error) -> Self {
    Self(serde_json::Value::String(format!("{error:#}")))
  }
}

impl<T: Serialize> From<T> for InvokeError {
  #[inline]
  fn from(value: T) -> Self {
    serde_json::to_value(value)
      .map(Self)
      .unwrap_or_else(Self::from_error)
  }
}

impl From<crate::Error> for InvokeError {
  #[inline(always)]
  fn from(error: crate::Error) -> Self {
    Self(serde_json::Value::String(error.to_string()))
  }
}

/// Response from a [`InvokeMessage`] passed to the [`InvokeResolver`].
#[derive(Debug)]
pub enum InvokeResponse {
  /// Resolve the promise.
  Ok(InvokeResponseBody),
  /// Reject the promise.
  Err(InvokeError),
}

impl<T: IpcResponse, E: Into<InvokeError>> From<Result<T, E>> for InvokeResponse {
  #[inline]
  fn from(result: Result<T, E>) -> Self {
    match result {
      Ok(ok) => match ok.body() {
        Ok(value) => Self::Ok(value),
        Err(err) => Self::Err(InvokeError::from_error(err)),
      },
      Err(err) => Self::Err(err.into()),
    }
  }
}

impl From<InvokeError> for InvokeResponse {
  fn from(error: InvokeError) -> Self {
    Self::Err(error)
  }
}

/// Resolver of a invoke message.
#[default_runtime(crate::Wry, wry)]
pub struct InvokeResolver<R: Runtime> {
  webview: Webview<R>,
  responder: Arc<Mutex<Option<Box<OwnedInvokeResponder<R>>>>>,
  cmd: String,
  pub(crate) callback: CallbackFn,
  pub(crate) error: CallbackFn,
}

impl<R: Runtime> Clone for InvokeResolver<R> {
  fn clone(&self) -> Self {
    Self {
      webview: self.webview.clone(),
      responder: self.responder.clone(),
      cmd: self.cmd.clone(),
      callback: self.callback,
      error: self.error,
    }
  }
}

impl<R: Runtime> InvokeResolver<R> {
  pub(crate) fn new(
    webview: Webview<R>,
    responder: Arc<Mutex<Option<Box<OwnedInvokeResponder<R>>>>>,
    cmd: String,
    callback: CallbackFn,
    error: CallbackFn,
  ) -> Self {
    Self {
      webview,
      responder,
      cmd,
      callback,
      error,
    }
  }

  /// Reply to the invoke promise with an async task.
  pub fn respond_async<T, F>(self, task: F)
  where
    T: IpcResponse,
    F: Future<Output = Result<T, InvokeError>> + Send + 'static,
  {
    crate::async_runtime::spawn(async move {
      Self::return_task(
        self.webview,
        self.responder,
        task,
        self.cmd,
        self.callback,
        self.error,
      )
      .await;
    });
  }

  /// Reply to the invoke promise with an async task which is already serialized.
  pub fn respond_async_serialized<F>(self, task: F)
  where
    F: Future<Output = Result<InvokeResponseBody, InvokeError>> + Send + 'static,
  {
    crate::async_runtime::spawn(async move {
      let response = match task.await {
        Ok(ok) => InvokeResponse::Ok(ok),
        Err(err) => InvokeResponse::Err(err),
      };
      Self::return_result(
        self.webview,
        self.responder,
        response,
        self.cmd,
        self.callback,
        self.error,
      )
    });
  }

  /// Reply to the invoke promise with a serializable value.
  pub fn respond<T: IpcResponse>(self, value: Result<T, InvokeError>) {
    Self::return_result(
      self.webview,
      self.responder,
      value.into(),
      self.cmd,
      self.callback,
      self.error,
    )
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<T: IpcResponse>(self, value: T) {
    self.respond(Ok(value))
  }

  /// Reject the invoke promise with a value.
  pub fn reject<T: Serialize>(self, value: T) {
    Self::return_result(
      self.webview,
      self.responder,
      Result::<(), _>::Err(value).into(),
      self.cmd,
      self.callback,
      self.error,
    )
  }

  /// Reject the invoke promise with an [`InvokeError`].
  pub fn invoke_error(self, error: InvokeError) {
    Self::return_result(
      self.webview,
      self.responder,
      error.into(),
      self.cmd,
      self.callback,
      self.error,
    )
  }

  /// Asynchronously executes the given task
  /// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
  ///
  /// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
  /// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
  pub async fn return_task<T, F>(
    webview: Webview<R>,
    responder: Arc<Mutex<Option<Box<OwnedInvokeResponder<R>>>>>,
    task: F,
    cmd: String,
    success_callback: CallbackFn,
    error_callback: CallbackFn,
  ) where
    T: IpcResponse,
    F: Future<Output = Result<T, InvokeError>> + Send + 'static,
  {
    let result = task.await;
    Self::return_closure(
      webview,
      responder,
      || result,
      cmd,
      success_callback,
      error_callback,
    )
  }

  pub(crate) fn return_closure<T: IpcResponse, F: FnOnce() -> Result<T, InvokeError>>(
    webview: Webview<R>,
    responder: Arc<Mutex<Option<Box<OwnedInvokeResponder<R>>>>>,
    f: F,
    cmd: String,
    success_callback: CallbackFn,
    error_callback: CallbackFn,
  ) {
    Self::return_result(
      webview,
      responder,
      f().into(),
      cmd,
      success_callback,
      error_callback,
    )
  }

  pub(crate) fn return_result(
    webview: Webview<R>,
    responder: Arc<Mutex<Option<Box<OwnedInvokeResponder<R>>>>>,
    response: InvokeResponse,
    cmd: String,
    success_callback: CallbackFn,
    error_callback: CallbackFn,
  ) {
    (responder.lock().unwrap().take().expect("resolver consumed"))(
      webview,
      cmd,
      response,
      success_callback,
      error_callback,
    );
  }
}

/// An invoke message.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct InvokeMessage<R: Runtime> {
  /// The webview that received the invoke message.
  pub(crate) webview: Webview<R>,
  /// Application managed state.
  pub(crate) state: Arc<StateManager>,
  /// The IPC command.
  pub(crate) command: String,
  /// The JSON argument passed on the invoke message.
  pub(crate) payload: InvokeBody,
  /// The request headers.
  pub(crate) headers: HeaderMap,
}

impl<R: Runtime> Clone for InvokeMessage<R> {
  fn clone(&self) -> Self {
    Self {
      webview: self.webview.clone(),
      state: self.state.clone(),
      command: self.command.clone(),
      payload: self.payload.clone(),
      headers: self.headers.clone(),
    }
  }
}

impl<R: Runtime> InvokeMessage<R> {
  /// Create an new [`InvokeMessage`] from a payload send by a webview.
  pub(crate) fn new(
    webview: Webview<R>,
    state: Arc<StateManager>,
    command: String,
    payload: InvokeBody,
    headers: HeaderMap,
  ) -> Self {
    Self {
      webview,
      state,
      command,
      payload,
      headers,
    }
  }

  /// The invoke command.
  #[inline(always)]
  pub fn command(&self) -> &str {
    &self.command
  }

  /// The webview that received the invoke.
  #[inline(always)]
  pub fn webview(&self) -> Webview<R> {
    self.webview.clone()
  }

  /// A reference to webview that received the invoke.
  #[inline(always)]
  pub fn webview_ref(&self) -> &Webview<R> {
    &self.webview
  }

  /// A reference to the payload the invoke received.
  #[inline(always)]
  pub fn payload(&self) -> &InvokeBody {
    &self.payload
  }

  /// The state manager associated with the application
  #[inline(always)]
  pub fn state(&self) -> Arc<StateManager> {
    self.state.clone()
  }

  /// A reference to the state manager associated with application.
  #[inline(always)]
  pub fn state_ref(&self) -> &StateManager {
    &self.state
  }

  /// The request headers.
  #[inline(always)]
  pub fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}

/// The `Callback` type is the return value of the `transformCallback` JavaScript function.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CallbackFn(pub u32);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_invoke_response_body() {
    let json = InvokeResponseBody::Json("[1, 123, 1231]".to_string());
    assert_eq!(json.deserialize::<Vec<u16>>().unwrap(), vec![1, 123, 1231]);

    let json = InvokeResponseBody::Json("\"string value\"".to_string());
    assert_eq!(json.deserialize::<String>().unwrap(), "string value");

    let json = InvokeResponseBody::Json("\"string value\"".to_string());
    assert!(json.deserialize::<Vec<u16>>().is_err());

    let values = vec![1, 2, 3, 4, 5, 6, 1];
    let raw = InvokeResponseBody::Raw(values.clone());
    assert_eq!(raw.deserialize::<Vec<u8>>().unwrap(), values);
  }
}
