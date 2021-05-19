use crate::cli::Args;
use std::process::Command;

// the name of the binary to find in $PATH
#[cfg(target_os = "linux")]
const DRIVER_BINARY: &str = "WebKitWebDriver";

#[cfg(target_os = "windows")]
const DRIVER_BINARY: &str = "./msedgedriver.exe";

// a prepared command of the native driver with necessary arguments
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn prepare_native_driver(args: &Args) -> Command {
  let mut cmd = Command::new(DRIVER_BINARY);
  cmd.arg(format!("--port={}", args.native_port));
  cmd
}

/// Find the native driver binary in the PATH, or exits the process with an error.
pub fn native(args: &Args) -> Command {
  if let Err(error) = which::which(DRIVER_BINARY) {
    eprintln!(
      "can not find binary {} in the PATH. This is currently required",
      DRIVER_BINARY
    );
    eprintln!("{:?}", error);
    std::process::exit(1);
  }

  let mut cmd = prepare_native_driver(args);
  cmd.env("TAURI_AUTOMATION_MODE", "TRUE");
  cmd
}
