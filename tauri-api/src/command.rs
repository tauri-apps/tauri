use std::process::{Child, Command, Stdio};

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

pub fn format_command(path: String, command: String) -> String {
  if cfg!(windows) {
    format!("{}/./{}.exe", path, command)
  } else {
    format!("{}/./{}", path, command)
  }
}

pub fn relative_command(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    Some(exe_dir) => return Ok(format_command(exe_dir.display().to_string(), command)),
    None => {
      return Err(crate::ErrorKind::Command("Could not evaluate executable dir".to_string()).into())
    }
  }
}

pub fn command_path(command: String) -> crate::Result<String> {
  match std::env::current_exe()?.parent() {
    #[cfg(not(windows))]
    Some(exe_dir) => Ok(format!("{}/{}", exe_dir.display().to_string(), command)),
    #[cfg(windows)]
    Some(exe_dir) => Ok(format!("{}/{}.exe", exe_dir.display().to_string(), command)),
    None => Err(crate::ErrorKind::Command("Could not evaluate executable dir".to_string()).into()),
  }
}

pub fn spawn_relative_command(
  command: String,
  args: Vec<String>,
  stdout: Stdio,
) -> crate::Result<Child> {
  let cmd = relative_command(command)?;
  Ok(Command::new(cmd).args(args).stdout(stdout).spawn()?)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{Error, ErrorKind};
  use totems::{assert_err, assert_ok};

  #[test]
  fn test_cmd_output() {
    let cmd = String::from("cat");

    let res = get_output(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    assert_ok!(&res);

    if let Ok(s) = &res {
      assert_eq!(*s, "This is a test doc!".to_string());
    }
  }

  #[test]
  fn test_cmd_fail() {
    let cmd = String::from("cat");

    let res = get_output(cmd, vec!["test/".to_string()], Stdio::piped());

    assert_err!(&res);

    if let Err(Error(ErrorKind::Command(e), _)) = &res {
      assert_eq!(*e, "cat: test/: Is a directory\n".to_string());
    }
  }

  #[test]
  fn check_relateive_cmd() {
    let cmd = String::from("cat");

    let res = relative_command(cmd.clone());

    assert_ok!(res);

    let current_exe = std::env::current_exe()
      .unwrap()
      .parent()
      .unwrap()
      .display()
      .to_string();

    if let Ok(s) = &res {
      assert_eq!(*s, format_command(current_exe, cmd));
    }
  }

  #[test]
  fn check_command_path() {
    let cmd = String::from("cat");

    let res = command_path(cmd);

    assert_ok!(res);
  }

  #[test]
  fn check_spawn_cmd() {
    let cmd = String::from("cat");

    let res = spawn_relative_command(cmd, vec!["test/test.txt".to_string()], Stdio::piped());

    assert_err!(&res);

    if let Err(Error(ErrorKind::Io(s), _)) = &res {
      assert_eq!(s.kind(), std::io::ErrorKind::NotFound);
    }
  }
}
