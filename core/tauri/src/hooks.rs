// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::ipc::{format_callback, format_callback_result},
  app::App,
  runtime::Runtime,
  StateManager, Window,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{future::Future, sync::Arc};

use tauri_macros::default_runtime;

/// A closure that is run when the Tauri application is setting up.
pub type SetupHook<R> =
  Box<dyn FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error + Send>> + Send>;

/// A closure that is run everytime Tauri receives a message it doesn't explicitly handle.
pub type InvokeHandler<R> = dyn Fn(Invoke<R>) + Send + Sync + 'static;

/// A closure that is responsible for respond a JS message.
pub type InvokeResponder<R> =
  dyn Fn(Window<R>, InvokeResponse, String, String) + Send + Sync + 'static;

/// A closure that is run once every time a window is created and loaded.
pub type OnPageLoad<R> = dyn Fn(Window<R>, PageLoadPayload) + Send + Sync + 'static;

/// The payload for the [`OnPageLoad`] hook.
#[derive(Debug, Clone, Deserialize)]
pub struct PageLoadPayload {
  url: String,
}

impl PageLoadPayload {
  /// The page URL.
  pub fn url(&self) -> &str {
    &self.url
  }
}

/// The message and resolver given to a custom command.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct Invoke<R: Runtime> {
  /// The message passed.
  pub message: InvokeMessage<R>,

  /// The resolver of the message.
  pub resolver: InvokeResolver<R>,
}

/// Error response from an [`InvokeMessage`].
#[derive(Debug)]
pub struct InvokeError(JsonValue);

impl InvokeError {
  /// Create an [`InvokeError`] as a string of the [`serde_json::Error`] message.
  #[inline(always)]
  pub fn from_serde_json(error: serde_json::Error) -> Self {
    Self(JsonValue::String(error.to_string()))
  }
}

impl<T: Serialize> From<T> for InvokeError {
  #[inline]
  fn from(value: T) -> Self {
    serde_json::to_value(value)
      .map(Self)
      .unwrap_or_else(Self::from_serde_json)
  }
}

impl From<crate::Error> for InvokeError {
  #[inline(always)]
  fn from(error: crate::Error) -> Self {
    Self(JsonValue::String(error.to_string()))
  }
}

/// Response from a [`InvokeMessage`] passed to the [`InvokeResolver`].
#[derive(Debug)]
pub enum InvokeResponse {
  /// Resolve the promise.
  Ok(JsonValue),
  /// Reject the promise.
  Err(InvokeError),
}

impl InvokeResponse {
  /// Turn a [`InvokeResponse`] back into a serializable result.
  #[inline(always)]
  pub fn into_result(self) -> Result<JsonValue, JsonValue> {
    match self {
      Self::Ok(v) => Ok(v),
      Self::Err(e) => Err(e.0),
    }
  }
}

impl<T: Serialize> From<Result<T, InvokeError>> for InvokeResponse {
  #[inline]
  fn from(result: Result<T, InvokeError>) -> Self {
    match result {
      Ok(ok) => match serde_json::to_value(ok) {
        Ok(value) => Self::Ok(value),
        Err(err) => Self::Err(InvokeError::from_serde_json(err)),
      },
      Err(err) => Self::Err(err),
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
#[derive(Debug)]
pub struct InvokeResolver<R: Runtime> {
  window: Window<R>,
  pub(crate) callback: String,
  pub(crate) error: String,
}

impl<R: Runtime> InvokeResolver<R> {
  pub(crate) fn new(window: Window<R>, callback: String, error: String) -> Self {
    Self {
      window,
      callback,
      error,
    }
  }

  /// Reply to the invoke promise with an async task.
  pub fn respond_async<T, F>(self, task: F)
  where
    T: Serialize,
    F: Future<Output = Result<T, InvokeError>> + Send + 'static,
  {
    crate::async_runtime::spawn(async move {
      self.return_task(task).await;
    });
  }

  /// Reply to the invoke promise with an async task which is already serialized.
  pub fn respond_async_serialized<F>(self, task: F)
  where
    F: Future<Output = Result<JsonValue, InvokeError>> + Send + 'static,
  {
    crate::async_runtime::spawn(async move {
      self.return_result(task.await.into());
    });
  }

  /// Reply to the invoke promise with a serializable value.
  pub fn respond<T: Serialize>(self, value: Result<T, InvokeError>) {
    self.return_result(value.into())
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<T: Serialize>(self, value: T) {
    self.return_result(Ok(value).into())
  }

  /// Reject the invoke promise with a value.
  pub fn reject<T: Serialize>(self, value: T) {
    self.return_result(Result::<(), _>::Err(value.into()).into())
  }

  /// Reject the invoke promise with an [`InvokeError`].
  pub fn invoke_error(self, error: InvokeError) {
    self.return_result(error.into())
  }

  /// Asynchronously executes the given task
  /// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
  ///
  /// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
  /// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
  pub async fn return_task<T, F>(self, task: F)
  where
    T: Serialize,
    F: Future<Output = Result<T, InvokeError>> + Send + 'static,
  {
    let result = task.await;
    self.return_closure(|| result)
  }

  pub(crate) fn return_closure<T: Serialize, F: FnOnce() -> Result<T, InvokeError>>(self, f: F) {
    self.return_result(f().into())
  }

  fn return_result(self, response: InvokeResponse) {
    (self.window.invoke_responder())(self.window, response, self.callback, self.error);
  }
}

pub fn window_invoke_responder<R: Runtime>(
  window: Window<R>,
  response: InvokeResponse,
  success_callback: String,
  error_callback: String,
) {
  let callback_string = match format_callback_result(
    response.into_result(),
    success_callback,
    error_callback.clone(),
  ) {
    Ok(callback_string) => callback_string,
    Err(e) => format_callback(error_callback, &e.to_string())
      .expect("unable to serialize shortcut string to json"),
  };

  let _ = window.eval(&callback_string);
}

/// An invoke message.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct InvokeMessage<R: Runtime> {
  /// The window that received the invoke message.
  pub(crate) window: Window<R>,
  /// Application managed state.
  pub(crate) state: Arc<StateManager>,
  /// The IPC command.
  pub(crate) command: String,
  /// The JSON argument passed on the invoke message.
  pub(crate) payload: JsonValue,
}

impl<R: Runtime> InvokeMessage<R> {
  /// Create an new [`InvokeMessage`] from a payload send to a window.
  pub(crate) fn new(
    window: Window<R>,
    state: Arc<StateManager>,
    command: String,
    payload: JsonValue,
  ) -> Self {
    Self {
      window,
      state,
      command,
      payload,
    }
  }

  /// The invoke command.
  #[inline(always)]
  pub fn command(&self) -> &str {
    &self.command
  }

  /// The window that received the invoke.
  #[inline(always)]
  pub fn window(&self) -> Window<R> {
    self.window.clone()
  }

  /// A reference to window that received the invoke.
  #[inline(always)]
  pub fn window_ref(&self) -> &Window<R> {
    &self.window
  }

  /// A reference to the payload the invoke received.
  #[inline(always)]
  pub fn payload(&self) -> &JsonValue {
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
}
