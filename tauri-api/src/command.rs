use std::process::{Child, Command, Stdio};

pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> Result<String, String> {
  Command::new(cmd)
    .args(args)
    .stdout(stdout)
    .output()
    .map_err(|err| err.to_string())
    .and_then(|output| {
      if output.status.success() {
        Result::Ok(String::from_utf8_lossy(&output.stdout).to_string())
      } else {
        Result::Err(String::from_utf8_lossy(&output.stderr).to_string())
      }
    })
}

pub fn format_command(path: String, command: String) -> String {
  if cfg!(windows) {
    format!("{}/./{}.exe", path, command)
  } else {
    format!("{}/./{}", path, command)
  }
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

pub fn command_path(command: String) -> Result<String, std::io::Error> {
  match std::env::current_exe()?.parent() {
    #[cfg(not(windows))]
    Some(exe_dir) => Ok(format!("{}/{}", exe_dir.display().to_string(), command)),
    #[cfg(windows)]
    Some(exe_dir) => Ok(format!("{}/{}.exe", exe_dir.display().to_string(), command)),
    None => Err(std::io::Error::new(
      std::io::ErrorKind::Other,
      "Could not evaluate executable dir".to_string(),
    )),
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

pub fn binary_command(
  binary_name: String
) -> Result<String, String> {
  return Ok(format!("{}-{}", binary_name, crate::platform::target_triple()?));
}