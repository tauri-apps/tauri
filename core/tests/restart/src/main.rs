// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::Env;

fn main() {
  let mut argv = std::env::args();
  let argc = argv.len();
  if argc == 0 || argc > 2 {
    panic!("restart test binary expect either no arguments or `restart`.")
  }

  println!(
    "{}",
    tauri::api::process::current_binary(&Default::default())
      .expect("tauri::api::process::current_binary could not resolve")
      .display()
  );

  match argv.nth(1).as_deref() {
    Some("restart") => {
      let mut env = Env::default();
      env.args.clear();
      tauri::api::process::restart(&env)
    }
    Some(invalid) => panic!("only argument `restart` is allowed, {} is invalid", invalid),
    None => {}
  };
}
