use std::{
  io::{BufRead, BufReader},
  process::{Command as StdCommand, Stdio},
  sync::Arc,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use crate::private::async_runtime::{channel, spawn, Receiver};
use os_pipe::pipe;
use serde::Serialize;
use shared_child::SharedChild;
use tauri_utils::platform;

/// A event sent to the command callback.
#[derive(Serialize)]
#[serde(tag = "event", content = "payload")]
pub enum CommandEvent {
  /// Stderr line.
  Stderr(String),
  /// Stdout line.
  Stdout(String),
  /// An error happened.
  Error(String),
  /// Finish status code.
  Finish(Option<i32>),
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
pub struct CommandChild(Arc<SharedChild>);

impl CommandChild {
  /// Send a kill signal to the child.
  pub fn kill(self) -> crate::Result<()> {
    self.0.kill()?;
    Ok(())
  }
}

impl Command {
  /// Creates a new Command for launching the given program.
  pub fn new<S: Into<String>>(program: S) -> Self {
    Self {
      program: program.into(),
      args: Default::default(),
    }
  }

  /// Creates a new Command for launching the given sidecar program.
  pub fn new_sidecar<S: Into<String>>(program: S) -> Self {
    Self::new(format!(
      "{}-{}",
      program.into(),
      platform::target_triple().expect("unsupported platform")
    ))
  }

  /// Append args to the command.
  pub fn args<I, S>(mut self, args: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    for arg in args {
      self.args.push(arg.as_ref().to_string());
    }
    self
  }

  /// Spawns the command.
  pub fn spawn(self) -> crate::Result<(Receiver<CommandEvent>, CommandChild)> {
    let mut command = get_std_command!(self);
    let (stdout_reader, stdout_writer) = pipe()?;
    let (stderr_reader, stderr_writer) = pipe()?;
    command.stdout(stdout_writer);
    command.stderr(stderr_writer);

    let shared_child = SharedChild::spawn(&mut command)?;
    let child = Arc::new(shared_child);
    let child_ = child.clone();

    let (tx, rx) = channel(1);
    let tx_ = tx.clone();
    spawn(async move {
      let _ = match child_.wait() {
        Ok(status) => tx_.send(CommandEvent::Finish(status.code())).await,
        Err(e) => tx_.send(CommandEvent::Error(e.to_string())).await,
      };
    });

    let tx_ = tx.clone();
    spawn(async move {
      let reader = BufReader::new(stdout_reader);
      for line in reader.lines() {
        let _ = match line {
          Ok(line) => tx_.send(CommandEvent::Stdout(line)).await,
          Err(e) => tx_.send(CommandEvent::Error(e.to_string())).await,
        };
      }
    });

    let tx_ = tx.clone();
    spawn(async move {
      let reader = BufReader::new(stderr_reader);
      for line in reader.lines() {
        let _ = match line {
          Ok(line) => tx_.send(CommandEvent::Stderr(line)).await,
          Err(e) => tx_.send(CommandEvent::Error(e.to_string())).await,
        };
      }
    });

    Ok((rx, CommandChild(child)))
  }
}

// tests for the commands functions.
#[cfg(test)]
mod test {
  use super::*;

  #[cfg(not(windows))]
  #[test]
  fn test_cmd_output() {
    // create a command to run cat.
    let cmd = Command::new("cat").args(&["test/test.txt"]);
    let (mut rx, _) = cmd.spawn().unwrap();

    crate::private::async_runtime::block_on(async move {
      while let Some(event) = rx.recv().await {
        match event {
          CommandEvent::Finish(code) => {
            assert_eq!(code, Some(0));
          }
          CommandEvent::Stdout(line) => {
            assert_eq!(line, "This is a test doc!".to_string());
          }
          _ => {}
        }
      }
    });
  }

  #[cfg(not(windows))]
  #[test]
  // test the failure case
  fn test_cmd_fail() {
    let cmd = Command::new("cat").args(&["test/"]);
    let (mut rx, _) = cmd.spawn().unwrap();

    crate::private::async_runtime::block_on(async move {
      while let Some(event) = rx.recv().await {
        match event {
          CommandEvent::Finish(code) => {
            assert_eq!(code, Some(1));
          }
          CommandEvent::Stderr(line) => {
            assert_eq!(line, "cat: test/: Is a directory".to_string());
          }
          _ => {}
        }
      }
    });
  }
}
