// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Cross-platform WebDriver server for Tauri applications.
//!
//! This is a [WebDriver Intermediary Node](https://www.w3.org/TR/webdriver/#dfn-intermediary-nodes) that wraps the native WebDriver server for platforms that [Tauri](https://github.com/tauri-apps/tauri) supports. Your WebDriver client will connect to the running `tauri-driver` server, and `tauri-driver` will handle starting the native WebDriver server for you behind the scenes. It requires two separate ports to be used since two distinct [WebDriver Remote Ends](https://www.w3.org/TR/webdriver/#dfn-remote-ends) run.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

#[cfg(any(target_os = "linux", windows))]
mod cli;
#[cfg(any(target_os = "linux", windows))]
mod server;
#[cfg(any(target_os = "linux", windows))]
mod webdriver;

// macOS scaffolding. Compiled but NOT wired into the runtime path: see
// `macos::MacOsDriver` doc-comment and `MACOS_DRIVER_DESIGN.md` for why this
// is staged in two PRs. The `cli` module is reused so the design + tests can
// pin the same arg shape as the other platforms. Dead-code is allowed here
// because the runtime path goes through the `eprintln!` branch in `main`
// below; the items are exercised only by the unit tests until the follow-up
// PR wires them in.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
mod cli;
#[cfg(target_os = "macos")]
#[allow(dead_code)]
mod macos;

#[cfg(target_os = "macos")]
fn main() {
  // Parse args so `--help` works the same on every platform; the rest of the
  // bridge is still TODO. Be honest about it instead of pretending to start.
  let _args: cli::Args = pico_args::Arguments::from_env().into();
  eprintln!(
    "tauri-driver: macOS support is currently a scaffold only. \
     See https://github.com/tauri-apps/tauri/issues/7068 and the \
     `tauri-driver` crate's `MACOS_DRIVER_DESIGN.md` for the planned approach. \
     Today no WebDriver session will be created."
  );
  std::process::exit(1);
}

#[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
fn main() {
  println!("tauri-driver is not supported on this platform");
  std::process::exit(1);
}

#[cfg(any(target_os = "linux", windows))]
fn main() {
  let args = pico_args::Arguments::from_env().into();

  #[cfg(windows)]
  let _job_handle = {
    let job = win32job::Job::create().unwrap();
    let mut info = job.query_extended_limit_info().unwrap();
    info.limit_kill_on_job_close();
    job.set_extended_limit_info(&info).unwrap();
    job.assign_current_process().unwrap();
    job
  };

  // start the native webdriver on the port specified in args
  let mut driver = webdriver::native(&args);
  let driver = driver
    .spawn()
    .expect("error while running native webdriver");

  // start our webdriver intermediary node
  if let Err(e) = server::run(args, driver) {
    eprintln!("error while running server: {e}");
    std::process::exit(1);
  }
}
