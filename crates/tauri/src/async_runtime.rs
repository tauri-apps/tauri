// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The singleton async runtime used by Tauri and exposed to users.
//!
//! Tauri uses [`tokio`] Runtime to initialize code, such as
//! [`Plugin::initialize`](../plugin/trait.Plugin.html#method.initialize) and [`crate::Builder::setup`] hooks.
//! This module also re-export some common items most developers need from [`tokio`]. If there's
//! one you need isn't here, you could use types in [`tokio`] directly.
//! For custom command handlers, it's recommended to use a plain `async fn` command.

pub use tokio::{
  runtime::{Handle as TokioHandle, Runtime as TokioRuntime},
  sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex, RwLock,
  },
  task::JoinHandle as TokioJoinHandle,
};

use std::{
  future::Future,
  pin::Pin,
  sync::OnceLock,
  task::{Context, Poll},
};

static RUNTIME: OnceLock<GlobalRuntime> = OnceLock::new();

struct GlobalRuntime {
  runtime: Option<Runtime>,
  handle: RuntimeHandle,
}

impl GlobalRuntime {
  fn handle(&self) -> RuntimeHandle {
    if let Some(r) = &self.runtime {
      r.handle()
    } else {
      self.handle.clone()
    }
  }

  fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    if let Some(r) = &self.runtime {
      r.spawn(task)
    } else {
      self.handle.spawn(task)
    }
  }

  pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    if let Some(r) = &self.runtime {
      r.spawn_blocking(func)
    } else {
      self.handle.spawn_blocking(func)
    }
  }

  fn block_on<F: Future>(&self, task: F) -> F::Output {
    if let Some(r) = &self.runtime {
      r.block_on(task)
    } else {
      self.handle.block_on(task)
    }
  }
}

/// A runtime used to execute asynchronous tasks.
pub enum Runtime {
  /// The tokio runtime.
  Tokio(TokioRuntime),
}

impl Runtime {
  /// Gets a reference to the [`TokioRuntime`].
  pub fn inner(&self) -> &TokioRuntime {
    let Self::Tokio(r) = self;
    r
  }

  /// Returns a handle of the async runtime.
  pub fn handle(&self) -> RuntimeHandle {
    match self {
      Self::Tokio(r) => RuntimeHandle::Tokio(r.handle().clone()),
    }
  }

  /// Spawns a future onto the runtime.
  pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    match self {
      Self::Tokio(r) => {
        let _guard = r.enter();
        JoinHandle::Tokio(tokio::spawn(task))
      }
    }
  }

  /// Runs the provided function on an executor dedicated to blocking operations.
  pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    match self {
      Self::Tokio(r) => JoinHandle::Tokio(r.spawn_blocking(func)),
    }
  }

  /// Runs a future to completion on runtime.
  pub fn block_on<F: Future>(&self, task: F) -> F::Output {
    match self {
      Self::Tokio(r) => r.block_on(task),
    }
  }
}

/// An owned permission to join on a task (await its termination).
#[derive(Debug)]
pub enum JoinHandle<T> {
  /// The tokio JoinHandle.
  Tokio(TokioJoinHandle<T>),
}

impl<T> JoinHandle<T> {
  /// Gets a reference to the [`TokioJoinHandle`].
  pub fn inner(&self) -> &TokioJoinHandle<T> {
    let Self::Tokio(t) = self;
    t
  }

  /// Abort the task associated with the handle.
  ///
  /// Awaiting a cancelled task might complete as usual if the task was
  /// already completed at the time it was cancelled, but most likely it
  /// will fail with a cancelled `JoinError`.
  pub fn abort(&self) {
    match self {
      Self::Tokio(t) => t.abort(),
    }
  }
}

impl<T> Future for JoinHandle<T> {
  type Output = crate::Result<T>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match self.get_mut() {
      Self::Tokio(t) => Pin::new(t).poll(cx).map_err(Into::into),
    }
  }
}

/// A handle to the async runtime
#[derive(Clone)]
pub enum RuntimeHandle {
  /// The tokio handle.
  Tokio(TokioHandle),
}

impl RuntimeHandle {
  /// Gets a reference to the [`TokioHandle`].
  pub fn inner(&self) -> &TokioHandle {
    let Self::Tokio(h) = self;
    h
  }

  /// Runs the provided function on an executor dedicated to blocking operations.
  pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    match self {
      Self::Tokio(h) => JoinHandle::Tokio(h.spawn_blocking(func)),
    }
  }

  /// Spawns a future onto the runtime.
  pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    match self {
      Self::Tokio(h) => {
        let _guard = h.enter();
        JoinHandle::Tokio(tokio::spawn(task))
      }
    }
  }

  /// Runs a future to completion on runtime.
  pub fn block_on<F: Future>(&self, task: F) -> F::Output {
    match self {
      Self::Tokio(h) => h.block_on(task),
    }
  }
}

fn default_runtime() -> GlobalRuntime {
  let runtime = Runtime::Tokio(TokioRuntime::new().unwrap());
  let handle = runtime.handle();
  GlobalRuntime {
    runtime: Some(runtime),
    handle,
  }
}

/// Sets the runtime to use to execute asynchronous tasks.
/// For convenience, this method takes a [`TokioHandle`].
/// Note that you cannot drop the underlying [`TokioRuntime`].
///
/// # Examples
///
/// ```rust
/// #[tokio::main]
/// async fn main() {
///   // perform some async task before initializing the app
///   do_something().await;
///   // share the current runtime with Tauri
///   tauri::async_runtime::set(tokio::runtime::Handle::current());
///
///   // bootstrap the tauri app...
///   // tauri::Builder::default().run().unwrap();
/// }
///
/// async fn do_something() {}
/// ```
///
/// # Panics
///
/// Panics if the runtime is already set.
pub fn set(handle: TokioHandle) {
  RUNTIME
    .set(GlobalRuntime {
      runtime: None,
      handle: RuntimeHandle::Tokio(handle),
    })
    .unwrap_or_else(|_| panic!("runtime already initialized"))
}

/// Returns a handle of the async runtime.
pub fn handle() -> RuntimeHandle {
  let runtime = RUNTIME.get_or_init(default_runtime);
  runtime.handle()
}

/// Runs a future to completion on runtime.
pub fn block_on<F: Future>(task: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(default_runtime);
  runtime.block_on(task)
}

/// Spawns a future onto the runtime.
pub fn spawn<F>(task: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(default_runtime);
  runtime.spawn(task)
}

/// Runs the provided function on an executor dedicated to blocking operations.
pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
where
  F: FnOnce() -> R + Send + 'static,
  R: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(default_runtime);
  runtime.spawn_blocking(func)
}

#[allow(dead_code)]
pub(crate) fn safe_block_on<F>(task: F) -> F::Output
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  if let Ok(handle) = tokio::runtime::Handle::try_current() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let handle_ = handle.clone();
    handle.spawn_blocking(move || {
      tx.send(handle_.block_on(task)).unwrap();
    });
    rx.recv().unwrap()
  } else {
    block_on(task)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn runtime_spawn() {
    let join = spawn(async { 5 });
    assert_eq!(join.await.unwrap(), 5);
  }

  #[test]
  fn runtime_block_on() {
    assert_eq!(block_on(async { 0 }), 0);
  }

  #[tokio::test]
  async fn handle_spawn() {
    let handle = handle();
    let join = handle.spawn(async { 5 });
    assert_eq!(join.await.unwrap(), 5);
  }

  #[test]
  fn handle_block_on() {
    let handle = handle();
    assert_eq!(handle.block_on(async { 0 }), 0);
  }

  #[tokio::test]
  async fn handle_abort() {
    let handle = handle();
    let join = handle.spawn(async {
      // Here we sleep 1 second to ensure this task to be uncompleted when abort() invoked.
      tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
      5
    });
    join.abort();
    if let crate::Error::JoinError(raw_error) = join.await.unwrap_err() {
      assert!(raw_error.is_cancelled());
    } else {
      panic!("Abort did not result in the expected `JoinError`");
    }
  }
}
