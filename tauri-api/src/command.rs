use shared_child::SharedChild;
use os_pipe::pipe;
use std::{
  process::{Child, Command as StdCommand, Stdio},
  sync::Arc,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use tauri_utils::platform;
use crate::private::async_runtime::{Sender, Receiver, channel, spawn};

/// A event sent to the command callback.
pub enum CommandEvent {
  /// Stderr line.
  Stderr(String),
  /// Stdout line.
  Stdout,
  /// Finish status code.
  Finish(u16),
}

macro_rules! get_std_command {
  ($self: ident) => {{
    let mut command = StdCommand::new($self.program);
    command.args(&$self.args);
    command.stdout(Stdio::piped());
    command.stdin(Stdio::piped());
    command.stderr(Stdio::piped());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
  }};
}

/// API to spawn commands.
pub struct Command {
  program: String,
  args: Vec<String>,
}

/// Child spawned.
pub struct CommandChild {
  /// Event receiver.
  pub event_rx: Receiver<CommandEvent>,
  inner: Arc<SharedChild>,
}

impl CommandChild {
  /// Send a kill signal to the child.
  pub fn kill(self) -> crate::Result<()> {
    self.inner.kill()?;
    Ok(())
  }
}

impl Command {
  /// Creates a new Command for launching the given program.
  pub fn new(program: String) -> Self {
    Self {
      program,
      args: Default::default(),
    }
  }

  /// Append args to the command.
  pub fn args(mut self, args: Vec<String>) -> Self {
    self.args.extend(args);
    self
  }

  /// Spawns the command.
  pub fn spawn(self) -> crate::Result<CommandChild> {
    let mut command = get_std_command!(self);
    let shared_child = SharedChild::spawn(&mut command)?;
    let child = Arc::new(shared_child);
    let child_ = child.clone();
    let (tx, rx) = channel(1);
    let tx_ = tx.clone();
    spawn(async move {
      match child_.wait() {
        Ok(status) => {}
        Err(e) => {}
      }
    });
    Ok(CommandChild {
      event_rx: rx,
      inner: child,
    })
  }
}

/// Gets the output of the given command.
#[cfg(not(windows))]
pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> crate::Result<String> {
  let output = StdCommand::new(cmd).args(args).stdout(stdout).output()?;

  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
  } else {
    Err(crate::Error::Command(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ))
  }
}

/// Gets the output of the given command.
#[cfg(windows)]
pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> crate::Result<String> {
  let output = StdCommand::new(cmd)
    .args(args)
    .stdout(stdout)
    .creation_flags(CREATE_NO_WINDOW)
    .output()?;

  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
  } else {
    Err(crate::Error::Command(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ))
  }
}

/// Gets the path to command relative to the current executable path.
#[cfg(not(windows))]
pub fn command_path(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => Ok(format!("{}/{}", exe_dir.display().to_string(), command)),
    None => Err(crate::Error::Command(
      "Could not evaluate executable dir".to_string(),
    )),
  }
}

/// Gets the path to command relative to the current executable path.
#[cfg(windows)]
pub fn command_path(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => Ok(format!("{}/{}.exe", exe_dir.display().to_string(), command)),
    None => Err(crate::Error::Command(
      "Could not evaluate executable dir".to_string(),
    )),
  }
}

/// Spawns a process with a command string relative to the current executable path.
/// For example, if your app bundles two executables, you don't need to worry about its path and just run `second-app`.
#[cfg(windows)]
pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> crate::Result<Child> {
  let cmd = command_path(command)?;
  Ok(
    StdCommand::new(cmd)
      .args(args)
      .creation_flags(CREATE_NO_WINDOW)
      .stdout(stdout)
      .spawn()?,
  )
}

/// Spawns a process with a command string relative to the current executable path.
/// For example, if your app bundles two executables, you don't need to worry about its path and just run `second-app`.
#[cfg(not(windows))]
pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> crate::Result<Child> {
  let cmd = command_path(command)?;
  Ok(StdCommand::new(cmd).args(args).stdout(stdout).spawn()?)
}

/// Gets the binary command with the current target triple.
pub fn binary_command(binary_name: String) -> crate::Result<String> {
  Ok(format!(
    "{}-{}",
    binary_name,
    platform::target_triple().map_err(|e| crate::Error::FailedToDetectPlatform(e.to_string()))?
  ))
}

// tests for the commands functions.
#[cfg(test)]
mod test {
  use super::*;

  #[cfg(not(windows))]
  #[test]
  // test the get_output function with a unix cat command.
  fn test_cmd_output() {
    // create a string with cat in it.
    let cmd = String::from("cat");

    // call get_output with cat and the argument test/test.txt on the stdio.
    let res = get_output(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    // assert that the result is an Ok() type
    assert!(res.is_ok());

    // if the assertion passes, assert the incoming data.
    if let Ok(s) = &res {
      // assert that cat returns the string in the test.txt document.
      assert_eq!(*s, "This is a test doc!".to_string());
    }
  }

  #[cfg(not(windows))]
  #[test]
  // test the failure case for get_output
  fn test_cmd_fail() {
    use crate::Error;

    // queue up a string with cat in it.
    let cmd = String::from("cat");

    // call get output with test/ as an argument on the stdio.
    let res = get_output(cmd, vec!["test/".to_string()], Stdio::piped());

    // assert that the result is an Error type.
    assert!(res.is_err());

    // destruct the Error to check the ErrorKind and test that it is a Command type.
    if let Error::Command(e) = res.unwrap_err() {
      // assert that the message in the error matches this string.
      assert_eq!(*e, "cat: test/: Is a directory\n".to_string());
    }
  }

  #[test]
  // test the command_path function
  fn check_command_path() {
    // generate a string for cat
    let cmd = String::from("cat");

    // call command_path on cat
    let res = command_path(cmd);

    // assert that the result is an OK() type.
    assert!(res.is_ok());
  }

  #[test]
  // check the spawn_relative_command function
  fn check_spawn_cmd() {
    // generate a cat string
    let cmd = String::from("cat");

    // call spawn_relative_command with cat and the argument test/test.txt on the Stdio.
    let res = spawn_relative_command(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    // this fails because there is no cat binary in the relative parent folder of this current executing command.
    assert!(res.is_err());

    // after asserting that the result is an error, check that the error kind is ErrorKind::Io
    if let crate::Error::Io(s) = res.unwrap_err() {
      // assert that the ErrorKind inside of the ErrorKind Io is ErrorKind::NotFound
      assert_eq!(s.kind(), std::io::ErrorKind::NotFound);
    }
  }
}
