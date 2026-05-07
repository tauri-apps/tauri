// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::{threadsafe_function::ThreadsafeFunction, Error, Result, Status};

#[cfg(not(target_family = "wasm"))]
#[napi_derive::napi]
pub fn run(
  args: Vec<String>,
  bin_name: Option<String>,
  callback: Arc<ThreadsafeFunction<bool>>,
) -> Result<()> {
  // we need to run in a separate thread so Node.js consumers
  // can do work while `tauri dev` is running.
  std::thread::spawn(move || {
    let res = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      tauri_cli::try_run(args, bin_name).inspect_err(|e| eprintln!("{e:#}"))
    })) {
      Ok(t) => t,
      Err(_) => {
        return callback.call(
          Err(Error::new(
            Status::GenericFailure,
            "Tauri CLI unexpected panic",
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
      }
    };

    match res {
      Ok(_) => callback.call(Ok(true), ThreadsafeFunctionCallMode::Blocking),
      Err(e) => callback.call(
        Err(Error::new(Status::GenericFailure, format!("{e:#}"))),
        ThreadsafeFunctionCallMode::Blocking,
      ),
    }
  });

  Ok(())
}

#[cfg(target_family = "wasm")]
#[napi_derive::napi]
pub fn run(
  _args: Vec<String>,
  _bin_name: Option<String>,
  _callback: Arc<ThreadsafeFunction<bool>>,
) -> Result<()> {
  Err(Error::new(
    Status::GenericFailure,
    "The Tauri CLI WebAssembly binding can be loaded in web/WASI environments, but running Tauri commands requires native toolchains and process spawning.",
  ))
}

#[napi_derive::napi]
pub fn log_error(error: String) {
  log::error!("{}", error);
}
