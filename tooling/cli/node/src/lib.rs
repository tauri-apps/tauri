// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Error, JsFunction, Result, Status,
};

#[napi_derive::napi]
pub fn run(args: Vec<String>, bin_name: Option<String>, callback: JsFunction) -> Result<()> {
  let function: ThreadsafeFunction<bool, ErrorStrategy::CalleeHandled> = callback
    .create_threadsafe_function(0, |ctx| ctx.env.get_boolean(ctx.value).map(|v| vec![v]))?;

  std::thread::spawn(move || match tauri_cli::run(args, bin_name) {
    Ok(_) => function.call(Ok(true), ThreadsafeFunctionCallMode::Blocking),
    Err(e) => function.call(
      Err(Error::new(Status::GenericFailure, format!("{:#}", e))),
      ThreadsafeFunctionCallMode::Blocking,
    ),
  });

  Ok(())
}
