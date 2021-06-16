// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cli;
mod server;
mod webdriver;

fn main() {
  let args = pico_args::Arguments::from_env().into();

  // start the native webdriver on the port specified in args
  let mut driver = webdriver::native(&args);
  driver
    .spawn()
    .expect("error while running native webdriver");

  // start our webdriver intermediary node
  server::run(args);
}
