// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;
use std::{
  collections::HashMap,
  path::PathBuf,
  process::{Command, Output, Stdio},
};

pub fn target_dir() -> PathBuf {
  let current_exe = std::env::current_exe().unwrap();
  let target_dir = current_exe.parent().unwrap().parent().unwrap();
  target_dir.into()
}

pub fn root_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_path_buf()
}

pub fn run_collect(cmd: &[&str]) -> (String, String) {
  let mut process_builder = Command::new(cmd[0]);
  process_builder
    .args(&cmd[1..])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  let prog = process_builder.spawn().expect("failed to spawn script");
  let Output {
    stdout,
    stderr,
    status,
  } = prog.wait_with_output().expect("failed to wait on child");
  let stdout = String::from_utf8(stdout).unwrap();
  let stderr = String::from_utf8(stderr).unwrap();
  if !status.success() {
    eprintln!("stdout: <<<{}>>>", stdout);
    eprintln!("stderr: <<<{}>>>", stderr);
    panic!("Unexpected exit code: {:?}", status.code());
  }
  (stdout, stderr)
}

pub fn parse_max_mem(output: &str) -> Option<u64> {
  // Takes the output from "time -v" as input and extracts the 'maximum
  // resident set size' and returns it in bytes.
  for line in output.lines() {
    if line
      .to_lowercase()
      .contains("maximum resident set size (kbytes)")
    {
      let value = line.split(": ").nth(1).unwrap();
      return Some(str::parse::<u64>(value).unwrap() * 1024);
    }
  }

  None
}

#[derive(Debug, Clone, Serialize)]
pub struct StraceOutput {
  pub percent_time: f64,
  pub seconds: f64,
  pub usecs_per_call: Option<u64>,
  pub calls: u64,
  pub errors: u64,
}

pub fn parse_strace_output(output: &str) -> HashMap<String, StraceOutput> {
  let mut summary = HashMap::new();

  let mut lines = output
    .lines()
    .filter(|line| !line.is_empty() && !line.contains("detached ..."));
  let count = lines.clone().count();

  if count < 4 {
    return summary;
  }

  let total_line = lines.next_back().unwrap();
  lines.next_back(); // Drop separator
  let data_lines = lines.skip(2);

  for line in data_lines {
    let syscall_fields = line.split_whitespace().collect::<Vec<_>>();
    let len = syscall_fields.len();
    let syscall_name = syscall_fields.last().unwrap();

    if (5..=6).contains(&len) {
      summary.insert(
        syscall_name.to_string(),
        StraceOutput {
          percent_time: str::parse::<f64>(syscall_fields[0]).unwrap(),
          seconds: str::parse::<f64>(syscall_fields[1]).unwrap(),
          usecs_per_call: Some(str::parse::<u64>(syscall_fields[2]).unwrap()),
          calls: str::parse::<u64>(syscall_fields[3]).unwrap(),
          errors: if syscall_fields.len() < 6 {
            0
          } else {
            str::parse::<u64>(syscall_fields[4]).unwrap()
          },
        },
      );
    }
  }

  let total_fields = total_line.split_whitespace().collect::<Vec<_>>();
  summary.insert(
    "total".to_string(),
    StraceOutput {
      percent_time: str::parse::<f64>(total_fields[0]).unwrap(),
      seconds: str::parse::<f64>(total_fields[1]).unwrap(),
      usecs_per_call: None,
      calls: str::parse::<u64>(total_fields[2]).unwrap(),
      errors: str::parse::<u64>(total_fields[3]).unwrap(),
    },
  );

  summary
}

pub fn run(cmd: &[&str]) {
  let mut process_builder = Command::new(cmd[0]);
  process_builder.args(&cmd[1..]).stdin(Stdio::piped());
  let mut prog = process_builder.spawn().expect("failed to spawn script");
  let status = prog.wait().expect("failed to wait on child");
  if !status.success() {
    panic!("Unexpected exit code: {:?}", status.code());
  }
}
