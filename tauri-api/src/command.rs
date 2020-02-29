use std::process::{Child, Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use tauri_utils::platform;

#[cfg(not(windows))]
pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> crate::Result<String> {
  Command::new(cmd)
    .args(args)
    .stdout(stdout)
    .output()
    .map_err(|err| crate::Error::with_chain(err, "Command: get output failed"))
    .and_then(|output| {
      if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
      } else {
        Err(crate::ErrorKind::Command(String::from_utf8_lossy(&output.stderr).to_string()).into())
      }
    })
}

#[cfg(windows)]
pub fn get_output(cmd: String, args: Vec<String>, stdout: Stdio) -> crate::Result<String> {
  Command::new(cmd)
    .args(args)
    .stdout(stdout)
    .creation_flags(CREATE_NO_WINDOW)
    .output()
    .map_err(|err| crate::Error::with_chain(err, "Command: get output failed"))
    .and_then(|output| {
      if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
      } else {
        Err(crate::ErrorKind::Command(String::from_utf8_lossy(&output.stderr).to_string()).into())
      }
    })
}

pub fn format_command(path: String, command: String) -> String {
  if cfg!(windows) {
    format!("{}/{}", path, command)
  } else {
    format!("{}/./{}", path, command)
  }
}

pub fn relative_command(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => Ok(format_command(exe_dir.display().to_string(), command)),
    None => Err(crate::ErrorKind::Command("Could not evaluate executable dir".to_string()).into()),
  }
}

pub fn command_path(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => Ok(format!("{}/{}", exe_dir.display().to_string(), command)),
    None => Err(crate::ErrorKind::Command("Could not evaluate executable dir".to_string()).into()),
  }
}

#[cfg(windows)]
pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> crate::Result<Child> {
  let cmd = relative_command(command)?;
  Ok(
    Command::new(cmd)
      .args(args)
      .creation_flags(CREATE_NO_WINDOW)
      .stdout(stdout)
      .spawn()?,
  )
}

#[cfg(not(windows))]
pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> crate::Result<Child> {
  let cmd = relative_command(command)?;
  Ok(Command::new(cmd).args(args).stdout(stdout).spawn()?)
}

pub fn binary_command(binary_name: String) -> crate::Result<String> {
  Ok(format!("{}-{}", binary_name, platform::target_triple()?))
}

// tests for the commands functions.
#[cfg(test)]
mod test {
  use super::*;
  use crate::{Error, ErrorKind};
  use totems::{assert_err, assert_ok};

  #[test]
  // test the get_output function with a unix cat command.
  fn test_cmd_output() {
    // create a string with cat in it.
    let cmd = String::from("cat");

    // call get_output with cat and the argument test/test.txt on the stdio.
    let res = get_output(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    // assert that the result is an Ok() type
    assert_ok!(&res);

    // if the assertion passes, assert the incoming data.
    if let Ok(s) = &res {
      // assert that cat returns the string in the test.txt document.
      assert_eq!(*s, "This is a test doc!".to_string());
    }
  }

  #[test]
  // test the failure case for get_output
  fn test_cmd_fail() {
    // queue up a string with cat in it.
    let cmd = String::from("cat");

    // call get output with test/ as an argument on the stdio.
    let res = get_output(cmd, vec!["test/".to_string()], Stdio::piped());

    // assert that the result is an Error type.
    assert_err!(&res);

    // destruct the Error to check the ErrorKind and test that it is a Command type.
    if let Err(Error(ErrorKind::Command(e), _)) = &res {
      // assert that the message in the error matches this string.
      assert_eq!(*e, "cat: test/: Is a directory\n".to_string());
    }
  }

  #[test]
  // test the relative_command function
  fn check_relateive_cmd() {
    // generate a cat string
    let cmd = String::from("cat");

    // call relative command on the cat string
    let res = relative_command(cmd.clone());

    // assert that the result comes back with Ok()
    assert_ok!(res);

    // get the parent directory of the current executable.
    let current_exe = std::env::current_exe()
      .unwrap()
      .parent()
      .unwrap()
      .display()
      .to_string();

    // check the string inside of the Ok
    if let Ok(s) = &res {
      // match the string against the call to format command with the current_exe.
      assert_eq!(*s, format_command(current_exe, cmd));
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
    assert_ok!(res);
  }

  #[test]
  // check the spawn_relative_command function
  fn check_spawn_cmd() {
    // generate a cat string
    let cmd = String::from("cat");

    // call spawn_relative_command with cat and the argument test/test.txt on the Stdio.
    let res = spawn_relative_command(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    // this fails because there is no cat binary in the relative parent folder of this current executing command.
    assert_err!(&res);

    // after asserting that the result is an error, check that the error kind is ErrorKind::Io
    if let Err(Error(ErrorKind::Io(s), _)) = &res {
      // assert that the ErrorKind inside of the ErrorKind Io is ErrorKind::NotFound
      assert_eq!(s.kind(), std::io::ErrorKind::NotFound);
    }
  }
}
