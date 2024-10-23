// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

const HELP: &str = "\
USAGE: tauri-driver [FLAGS] [OPTIONS]

FLAGS:
  -h, --help              Prints help information

OPTIONS:
  --port NUMBER           Sets the tauri-driver intermediary port
  --native-port NUMBER    Sets the port of the underlying WebDriver
  --native-host HOST      Sets the host of the underlying WebDriver (Linux only)
  --native-driver PATH    Sets the path to the native WebDriver binary
";

#[derive(Debug, Clone)]
pub struct Args {
  pub port: u16,
  pub native_port: u16,
  pub native_host: String,
  pub native_driver: Option<PathBuf>,
}

impl From<pico_args::Arguments> for Args {
  fn from(mut args: pico_args::Arguments) -> Self {
    // if the user wanted help, we don't care about parsing the rest of the args
    if args.contains(["-h", "--help"]) {
      println!("{}", HELP);
      std::process::exit(0);
    }

    let native_driver = match args.opt_value_from_str("--native-driver") {
      Ok(native_driver) => native_driver,
      Err(e) => {
        eprintln!("Error while parsing option --native-driver: {}", e);
        std::process::exit(1);
      }
    };

    let parsed = Args {
      port: args.value_from_str("--port").unwrap_or(4444),
      native_port: args.value_from_str("--native-port").unwrap_or(4445),
      native_host: args
        .value_from_str("--native-host")
        .unwrap_or(String::from("127.0.0.1")),
      native_driver,
    };

    // be strict about accepting args, error for anything extraneous
    let rest = args.finish();
    if !rest.is_empty() {
      eprintln!("Error: unused arguments left: {:?}", rest);
      eprintln!("{}", HELP);
      std::process::exit(1);
    }

    parsed
  }
}
