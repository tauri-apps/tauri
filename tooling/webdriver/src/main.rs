// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/tauri/raw/dev/.github/splash.png)](https://tauri.app)
//!
//! Cross-platform WebDriver server for Tauri applications.
//!
//! This is a [WebDriver Intermediary Node](https://www.w3.org/TR/webdriver/#dfn-intermediary-nodes) that wraps the native WebDriver server for platforms that [Tauri](https://github.com/tauri-apps/tauri) supports. Your WebDriver client will connect to the running `tauri-driver` server, and `tauri-driver` will handle starting the native WebDriver server for you behind the scenes. It requires two separate ports to be used since two distinct [WebDriver Remote Ends](https://www.w3.org/TR/webdriver/#dfn-remote-ends) run.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

mod cli;
mod server;
mod webdriver;

fn main() {
  let args = pico_args::Arguments::from_env().into();

  #[cfg(windows)]
  let _ = {
    use win32job::Job;
    let job = Job::create().unwrap();
    let mut info = job.query_extended_limit_info().unwrap();
    job.set_extended_limit_info(&mut info).unwrap();
    job.assign_current_process().unwrap();
  };

  // start the native webdriver on the port specified in args
  let mut driver = webdriver::native(&args);
  let driver = driver
    .spawn()
    .expect("error while running native webdriver");

  // start our webdriver intermediary node
  if let Err(e) = server::run(args, driver) {
    eprintln!("error while running server: {}", e);
    std::process::exit(1);
  }
}
