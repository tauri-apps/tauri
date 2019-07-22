use proton_ui::WebView;

use std::process::{Child, Command, Stdio};

use super::run_async;

pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> Result<String, String> {
  Command::new(cmd)
    .args(args)
    .stdout(stdout)
    .output()
    .map_err(|err| err.to_string())
    .and_then(|output| {
      if output.status.success() {
        return Result::Ok(String::from_utf8_lossy(&output.stdout).to_string());
      } else {
        return Result::Err(String::from_utf8_lossy(&output.stderr).to_string());
      }
    })
}

// TODO use .exe for windows builds
pub fn format_command(path: String, command: String) -> String {
  return format!("{}/./{}", path, command);
}

pub fn relative_command(command: String) -> Result<String, std::io::Error> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => return Ok(format_command(exe_dir.display().to_string(), command)),
    None => {
      return Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not evaluate executable dir".to_string(),
      ))
    }
  }
}

// TODO append .exe for windows builds
pub fn command_path(command: String) -> Result<String, std::io::Error> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => return Ok(format!("{}/{}", exe_dir.display().to_string(), command)),
    None => {
      return Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not evaluate executable dir".to_string(),
      ))
    }
  }
}

pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> Result<Child, std::io::Error> {
  let cmd = relative_command(command)?;
  Ok(Command::new(cmd).args(args).stdout(stdout).spawn()?)
}

pub fn call<T: 'static>(
  webview: &mut WebView<'_, T>,
  command: String,
  args: Vec<String>,
  callback: String,
  error: String,
) {
  run_async(
    webview,
    || {
      get_output(command, args, Stdio::piped())
        .map_err(|err| format!("`{}`", err))
        .map(|output| format!("`{}`", output))
    },
    callback,
    error,
  );
}
