// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(target_os = "macos", target_os = "linux", windows))]

use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Error, JsFunction, Result, Status,
};

#[napi_derive::napi]
pub fn run(args: Vec<String>, bin_name: Option<String>, callback: JsFunction) -> Result<()> {
  let function: ThreadsafeFunction<bool, ErrorStrategy::CalleeHandled> = callback
    .create_threadsafe_function(0, |ctx| ctx.env.get_boolean(ctx.value).map(|v| vec![v]))?;

  // we need to run in a separate thread so Node.js consumers
  // can do work while `tauri dev` is running.
  std::thread::spawn(move || {
    let res = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      tauri_cli::try_run(args, bin_name).inspect_err(|e| eprintln!("{e:#}"))
    })) {
      Ok(t) => t,
      Err(_) => {
        return function.call(
          Err(Error::new(
            Status::GenericFailure,
            "Tauri CLI unexpected panic",
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
      }
    };

    match res {
      Ok(_) => function.call(Ok(true), ThreadsafeFunctionCallMode::Blocking),
      Err(e) => function.call(
        Err(Error::new(Status::GenericFailure, format!("{:#}", e))),
        ThreadsafeFunctionCallMode::Blocking,
      ),
    }
  });

  Ok(())
}

#[napi_derive::napi]
pub fn log_error(error: String) {
  log::error!("{}", error);
}
