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
pub type SetupHook<M> = Box<dyn Fn(&mut App<M>) -> Result<(), Box<dyn std::error::Error>> + Send>;

/// A closure that is run everytime Tauri receives a message it doesn't explicitly handle.
pub type InvokeHandler<M> = dyn Fn(InvokeMessage<M>, InvokeResolver<M>) + Send + Sync + 'static;

/// A closure that is run once every time a window is created and loaded.
pub type OnPageLoad<M> = dyn Fn(Window<M>, PageLoadPayload) + Send + Sync + 'static;

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

/// Response from a [`InvokeMessage`] passed to the [`InvokeResolver`].
#[derive(Debug)]
pub enum InvokeResponse {
  /// Resolve the promise.
  Ok(JsonValue),
  /// Reject the promise.
  Err(JsonValue),
}

impl<T: Serialize, E: Serialize> From<Result<T, E>> for InvokeResponse {
  fn from(result: Result<T, E>) -> Self {
    match result {
      Result::Ok(t) => match serde_json::to_value(t) {
        Ok(v) => Self::Ok(v),
        Err(e) => Self::Err(JsonValue::String(e.to_string())),
      },
      Result::Err(e) => Self::error(e),
    }
  }
}

impl InvokeResponse {
  #[doc(hidden)]
  pub fn error<T: Serialize>(value: T) -> Self {
    match serde_json::to_value(value) {
      Ok(v) => Self::Err(v),
      Err(e) => Self::Err(JsonValue::String(e.to_string())),
    }
  }
}

/// Resolver of a invoke message.
pub struct InvokeResolver<M: Params> {
  window: Window<M>,
  pub(crate) main_thread: bool,
  pub(crate) callback: String,
  pub(crate) error: String,
}

/*impl<P: Params> Clone for InvokeResolver<P> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      main_thread: self.main_thread,
      callback: self.callback.clone(),
      error: self.error.clone(),
    }
  }
}*/

impl<M: Params> InvokeResolver<M> {
  pub(crate) fn new(window: Window<M>, main_thread: bool, callback: String, error: String) -> Self {
    Self {
      window,
      main_thread,
      callback,
      error,
    }
  }

  /// Reply to the invoke promise with an async task.
  pub fn respond_async<F: Future<Output = InvokeResponse> + Send + 'static>(self, task: F) {
    if self.main_thread {
      crate::async_runtime::block_on(async move {
        Self::return_task(self.window, task, self.callback, self.error).await;
      });
    } else {
      crate::async_runtime::spawn(async move {
        Self::return_task(self.window, task, self.callback, self.error).await;
      });
    }
  }

  /// Reply to the invoke promise running the given closure.
  pub fn respond_closure<F: FnOnce() -> InvokeResponse>(self, f: F) {
    Self::return_closure(self.window, f, self.callback, self.error)
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<S, ()>::Ok(value).into(),
      self.callback,
      self.error,
    )
  }

  /// Reject the invoke promise with a value.
  pub fn reject<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<(), S>::Err(value).into(),
      self.callback,
      self.error,
    )
  }

  /// Asynchronously executes the given task
  /// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
  ///
  /// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
  /// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
  pub async fn return_task<F: std::future::Future<Output = InvokeResponse> + Send + 'static>(
    window: Window<M>,
    task: F,
    success_callback: String,
    error_callback: String,
  ) {
    let result = task.await;
    Self::return_closure(window, || result, success_callback, error_callback)
  }

  pub(crate) fn return_closure<F: FnOnce() -> InvokeResponse>(
    window: Window<M>,
    f: F,
    success_callback: String,
    error_callback: String,
  ) {
    Self::return_result(window, f(), success_callback, error_callback)
  }

  pub(crate) fn return_result(
    window: Window<M>,
    response: InvokeResponse,
    success_callback: String,
    error_callback: String,
  ) {
    let callback_string = match format_callback_result(
      match response {
        InvokeResponse::Ok(t) => std::result::Result::Ok(t),
        InvokeResponse::Err(e) => std::result::Result::Err(e),
      },
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

impl<M: Params> InvokeMessage<M> {
  /// Create an new [`InvokeMessage`] from a payload send to a window.
  pub(crate) fn new(
    window: Window<M>,
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
  pub fn command(&self) -> &str {
    &self.command
  }

  /// The window that received the invoke.
  pub fn window(&self) -> Window<M> {
    self.window.clone()
  }
}
