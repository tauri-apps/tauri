// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The singleton async runtime used by Tauri and exposed to consumers.
//! Wraps a `tokio` Runtime and is meant to be used by initialization code, such as plugins `initialization` and app `setup` hooks.
//! Fox more complex use cases, consider creating your own runtime.
//! For command handlers, it's recommended to use a plain `async fn` command.

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
pub use tokio::sync::{
  mpsc::{channel, Receiver, Sender},
  Mutex, RwLock,
};

use std::future::Future;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// Run a future to completion on runtime.
pub fn block_on<F: Future>(task: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
  runtime.block_on(task)
}

/// Spawn a future onto the runtime.
pub fn spawn<F>(task: F)
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
  runtime.spawn(task);
}
