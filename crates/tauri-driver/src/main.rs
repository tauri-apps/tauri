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

#[cfg(not(any(target_os = "linux", windows)))]
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
  let mut driver = driver
    .spawn()
    .expect("error while running native webdriver");

  // wait until the native webdriver accepts connections, so that we never
  // accept client connections we cannot serve yet
  if let Err(e) = wait_for_native_driver(
    &mut driver,
    &args.native_host,
    args.native_port,
    std::time::Duration::from_secs(30),
  ) {
    eprintln!("error while waiting for the native webdriver to start: {e}");
    let _ = driver.kill();
    std::process::exit(1);
  }

  // start our webdriver intermediary node
  if let Err(e) = server::run(args, driver) {
    eprintln!("error while running server: {e}");
    std::process::exit(1);
  }
}

#[cfg(any(target_os = "linux", windows))]
fn wait_for_native_driver(
  driver: &mut std::process::Child,
  host: &str,
  port: u16,
  timeout: std::time::Duration,
) -> std::io::Result<()> {
  use std::io::{Error, ErrorKind};

  let start = std::time::Instant::now();
  loop {
    if let Some(status) = driver.try_wait()? {
      return Err(Error::other(format!(
        "native webdriver exited before accepting connections: {status}"
      )));
    }

    match std::net::TcpStream::connect((host, port)) {
      Ok(_) => return Ok(()),
      Err(e) if matches!(e.kind(), ErrorKind::ConnectionRefused | ErrorKind::TimedOut) => {}
      Err(e) => return Err(e),
    }

    if start.elapsed() >= timeout {
      return Err(Error::new(
        ErrorKind::TimedOut,
        format!("native webdriver did not accept connections on {host}:{port} within {timeout:?}"),
      ));
    }

    std::thread::sleep(std::time::Duration::from_millis(100));
  }
}
