// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::rpc::{format_callback, format_callback_result},
  runtime::app::App,
  Params, StateManager, Window,
};
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};

/// A closure that is run when the Tauri application is setting up.
pub type SetupHook<M> = Box<dyn Fn(&mut App<M>) -> Result<(), Box<dyn std::error::Error>> + Send>;

/// A closure that is run everytime Tauri receives a message it doesn't explicitly handle.
pub type InvokeHandler<M> = dyn Fn(InvokeMessage<M>, Arc<StateManager>) + Send + Sync + 'static;

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

/// Payload from an invoke call.
#[derive(Debug, Deserialize)]
pub(crate) struct InvokePayload {
  #[serde(rename = "__tauriModule")]
  pub(crate) tauri_module: Option<String>,
  pub(crate) callback: String,
  pub(crate) error: String,
  #[serde(rename = "mainThread", default)]
  pub(crate) main_thread: bool,
  #[serde(flatten)]
  pub(crate) inner: serde_json::Value,
}

/// An invoke message.
pub struct InvokeMessage<M: Params> {
  window: Window<M>,
  pub(crate) command: String,

  /// Allow our crate to access the payload without cloning it.
  pub(crate) payload: InvokePayload,
}

impl<M: Params> InvokeMessage<M> {
  /// Create an new [`InvokeMessage`] from a payload send to a window.
  pub(crate) fn new(window: Window<M>, command: String, payload: InvokePayload) -> Self {
    Self {
      window,
      command,
      payload,
    }
  }

  /// The invoke command.
  pub fn command(&self) -> &str {
    &self.command
  }

  /// The invoke payload.
  pub fn payload(&self) -> serde_json::Value {
    self.payload.inner.clone()
  }

  /// The window that received the invoke.
  pub fn window(&self) -> Window<M> {
    self.window.clone()
  }

  /// Reply to the invoke promise with an async task.
  pub fn respond_async<
    T: Serialize,
    Err: Serialize,
    F: Future<Output = Result<T, Err>> + Send + 'static,
  >(
    self,
    task: F,
  ) {
    if self.payload.main_thread {
      crate::async_runtime::block_on(async move {
        Self::return_task(self.window, task, self.payload.callback, self.payload.error).await;
      });
    } else {
      crate::async_runtime::spawn(async move {
        Self::return_task(self.window, task, self.payload.callback, self.payload.error).await;
      });
    }
  }

  /// Reply to the invoke promise running the given closure.
  pub fn respond_closure<T: Serialize, Err: Serialize, F: FnOnce() -> Result<T, Err>>(self, f: F) {
    Self::return_closure(self.window, f, self.payload.callback, self.payload.error)
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<S, ()>::Ok(value),
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Reject the invoke promise with a value.
  pub fn reject<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<(), S>::Err(value),
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Asynchronously executes the given task
  /// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
  ///
  /// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
  /// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
  pub async fn return_task<
    T: Serialize,
    Err: Serialize,
    F: std::future::Future<Output = Result<T, Err>> + Send + 'static,
  >(
    window: Window<M>,
    task: F,
    success_callback: String,
    error_callback: String,
  ) {
    let result = task.await;
    Self::return_closure(window, || result, success_callback, error_callback)
  }

  pub(crate) fn return_closure<T: Serialize, Err: Serialize, F: FnOnce() -> Result<T, Err>>(
    window: Window<M>,
    f: F,
    success_callback: String,
    error_callback: String,
  ) {
    Self::return_result(window, f(), success_callback, error_callback)
  }

  pub(crate) fn return_result<T: Serialize, Err: Serialize>(
    window: Window<M>,
    result: Result<T, Err>,
    success_callback: String,
    error_callback: String,
  ) {
    let callback_string =
      match format_callback_result(result, success_callback, error_callback.clone()) {
        Ok(callback_string) => callback_string,
        Err(e) => format_callback(error_callback, &e.to_string())
          .expect("unable to serialize shortcut string to json"),
      };

    let _ = window.eval(&callback_string);
  }
}
