// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::rpc::{format_callback, format_callback_result},
  runtime::app::App,
  Params, StateManager, Window,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{future::Future, sync::Arc};

/// A closure that is run when the Tauri application is setting up.
pub type SetupHook<P> = Box<dyn Fn(&mut App<P>) -> Result<(), Box<dyn std::error::Error>> + Send>;

/// A closure that is run everytime Tauri receives a message it doesn't explicitly handle.
pub type InvokeHandler<P> = dyn Fn(Invoke<P>) + Send + Sync + 'static;

/// A closure that is run once every time a window is created and loaded.
pub type OnPageLoad<P> = dyn Fn(Window<P>, PageLoadPayload) + Send + Sync + 'static;

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
pub struct Invoke<P: Params> {
  /// The message passed.
  pub message: InvokeMessage<P>,

  /// The resolver of the message.
  pub resolver: InvokeResolver<P>,
}

/// Error response from an [`InvokeMessage`].
#[derive(Debug)]
pub struct InvokeError(JsonValue);

impl InvokeError {
  /// Create an [`InvokeError`] as a string of the [`serde_json::Error`] message.
  pub fn from_serde_json(error: serde_json::Error) -> Self {
    Self(JsonValue::String(error.to_string()))
  }
}

impl<T: Serialize> From<T> for InvokeError {
  fn from(value: T) -> Self {
    serde_json::to_value(value)
      .map(Self)
      .unwrap_or_else(Self::from_serde_json)
  }
}

impl From<crate::Error> for InvokeError {
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
  pub fn into_result(self) -> Result<JsonValue, JsonValue> {
    match self {
      Self::Ok(v) => Ok(v),
      Self::Err(e) => Err(e.0),
    }
  }
}

impl<T: Serialize> From<Result<T, InvokeError>> for InvokeResponse {
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

/// Resolver of a invoke message.
pub struct InvokeResolver<M: Params> {
  window: Window<M>,
  pub(crate) callback: String,
  pub(crate) error: String,
}

impl<P: Params> InvokeResolver<P> {
  pub(crate) fn new(window: Window<P>, callback: String, error: String) -> Self {
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
      Self::return_task(self.window, task, self.callback, self.error).await;
    });
  }

  /// Reply to the invoke promise running the given closure.
  pub fn respond_closure<T, F>(self, f: F)
  where
    T: Serialize,
    F: FnOnce() -> Result<T, InvokeError>,
  {
    Self::return_closure(self.window, f, self.callback, self.error)
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<S: Serialize>(self, value: S) {
    Self::return_result(self.window, Ok(value), self.callback, self.error)
  }

  /// Reject the invoke promise with a value.
  pub fn reject<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<(), _>::Err(value.into()),
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
    window: Window<P>,
    task: F,
    success_callback: String,
    error_callback: String,
  ) where
    T: Serialize,
    F: Future<Output = Result<T, InvokeError>> + Send + 'static,
  {
    let result = task.await;
    Self::return_closure(window, || result, success_callback, error_callback)
  }

  pub(crate) fn return_closure<T: Serialize, F: FnOnce() -> Result<T, InvokeError>>(
    window: Window<P>,
    f: F,
    success_callback: String,
    error_callback: String,
  ) {
    Self::return_result(window, f(), success_callback, error_callback)
  }

  pub(crate) fn return_result<T: Serialize>(
    window: Window<P>,
    response: Result<T, InvokeError>,
    success_callback: String,
    error_callback: String,
  ) {
    let callback_string = match format_callback_result(
      InvokeResponse::from(response).into_result(),
      success_callback,
      error_callback.clone(),
    ) {
      Ok(callback_string) => callback_string,
      Err(e) => format_callback(error_callback, &e.to_string())
        .expect("unable to serialize shortcut string to json"),
    };

    let _ = window.eval(&callback_string);
  }
}

/// An invoke message.
pub struct InvokeMessage<M: Params> {
  /// The window that received the invoke message.
  pub(crate) window: Window<M>,
  /// Application managed state.
  pub(crate) state: Arc<StateManager>,
  /// The RPC command.
  pub(crate) command: String,
  /// The JSON argument passed on the invoke message.
  pub(crate) payload: JsonValue,
}

impl<P: Params> InvokeMessage<P> {
  /// Create an new [`InvokeMessage`] from a payload send to a window.
  pub(crate) fn new(
    window: Window<P>,
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
  pub fn window(&self) -> Window<P> {
    self.window.clone()
  }

  /// A reference to window that received the invoke.
  #[inline(always)]
  pub fn window_ref(&self) -> &Window<P> {
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
