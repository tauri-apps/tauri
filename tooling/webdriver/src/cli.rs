const HELP: &str = "\
USAGE: tauri-driver [FLAGS] [OPTIONS]

FLAGS:
  -h, --help              Prints help information

OPTIONS:
  --port NUMBER           Sets the tauri-driver intermediary port
  --native-port NUMBER    Sets the port of the underlying webdriver
";

#[derive(Debug, Copy, Clone)]
pub struct Args {
  pub port: u16,
  pub native_port: u16,
}

impl From<pico_args::Arguments> for Args {
  fn from(mut args: pico_args::Arguments) -> Self {
    // if the user wanted help, we don't care about parsing the rest of the args
    if args.contains(["-h", "--help"]) {
      println!("{}", HELP);
      std::process::exit(0);
    }

    let parsed = Args {
      port: args.value_from_str("--port").unwrap_or(4444),
      native_port: args.value_from_str("--native-port").unwrap_or(4445),
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
