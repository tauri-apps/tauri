// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::net::TcpListener;

use rand::distributions::{Distribution, Uniform};

/// Gets the first available port between 8000 and 9000.
pub fn get_available_port() -> Option<u16> {
  let mut rng = rand::thread_rng();
  let die = Uniform::from(8000..9000);

  for _i in 0..100 {
    let port = die.sample(&mut rng);
    if port_is_available(port) {
      return Some(port);
    }
  }
  None
}

/// Checks if the given port is available to use.
pub fn port_is_available(port: u16) -> bool {
  TcpListener::bind(("127.0.0.1", port)).is_ok()
}
