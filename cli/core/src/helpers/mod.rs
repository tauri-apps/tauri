pub mod app_paths;
pub mod config;
mod logger;
pub mod manifest;
pub mod updater_signature;

pub use logger::Logger;

use std::{
  io::{BufRead, BufReader},
  process::{Command, Stdio},
};

pub fn execute_with_output(cmd: &mut Command) -> crate::Result<()> {
  let mut child = cmd
    .stdout(Stdio::piped())
    .spawn()
    .expect("failed to spawn command");
  {
    let stdout = child.stdout.as_mut().expect("Failed to get stdout handle");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      println!("{}", line.expect("Failed to get line"));
    }
  }

  let status = child.wait()?;
  if status.success() {
    Ok(())
  } else {
    Err(anyhow::anyhow!("command failed"))
  }
}
