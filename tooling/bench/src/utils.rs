// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::HashMap,
  fs,
  io::{BufRead, BufReader},
  path::PathBuf,
  process::{Command, Output, Stdio},
};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct BenchResult {
  pub created_at: String,
  pub sha1: String,
  pub exec_time: HashMap<String, HashMap<String, f64>>,
  pub binary_size: HashMap<String, u64>,
  pub max_memory: HashMap<String, u64>,
  pub thread_count: HashMap<String, u64>,
  pub syscall_count: HashMap<String, u64>,
  pub cargo_deps: HashMap<String, usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct StraceOutput {
  pub percent_time: f64,
  pub seconds: f64,
  pub usecs_per_call: Option<u64>,
  pub calls: u64,
  pub errors: u64,
}

pub fn get_target() -> &'static str {
  #[cfg(target_os = "macos")]
  return "x86_64-apple-darwin";
  #[cfg(target_os = "linux")]
  return "x86_64-unknown-linux-gnu";
  #[cfg(target_os = "windows")]
  return unimplemented!();
}

pub fn target_dir() -> PathBuf {
  let target_dir = bench_root_path()
    .join("tests")
    .join("target")
    .join(get_target())
    .join("release");
  target_dir.into()
}

pub fn bench_root_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[allow(dead_code)]
pub fn home_path() -> PathBuf {
  #[cfg(any(target_os = "macos", target_os = "linux"))]
  return PathBuf::from(env!("HOME"));
  #[cfg(any(target_os = "windows"))]
  return PathBuf::from(env!("HOMEPATH"));
}

#[allow(dead_code)]
pub fn tauri_root_path() -> PathBuf {
  bench_root_path()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_path_buf()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn parse_max_mem(file_path: &str) -> Option<u64> {
  let file = fs::File::open(file_path).unwrap();
  let output = BufReader::new(file);
  let mut highest: u64 = 0;
  // MEM 203.437500 1621617192.4123
  for line in output.lines() {
    if let Ok(line) = line {
      // split line by space
      let split = line.split(" ").collect::<Vec<_>>();
      if split.len() == 3 {
        // mprof generate result in MB
        let current_bytes = str::parse::<f64>(split[1]).unwrap() as u64 * 1024 * 1024;
        if current_bytes > highest {
          highest = current_bytes;
        }
      }
    }
  }

  fs::remove_file(file_path).unwrap();

  if highest > 0 {
    return Some(highest);
  }

  None
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn run(cmd: &[&str]) {
  let mut process_builder = Command::new(cmd[0]);
  process_builder.args(&cmd[1..]).stdin(Stdio::piped());
  let mut prog = process_builder.spawn().expect("failed to spawn script");
  let status = prog.wait().expect("failed to wait on child");
  if !status.success() {
    panic!("Unexpected exit code: {:?}", status.code());
  }
}

#[allow(dead_code)]
pub fn read_json(filename: &str) -> Result<Value> {
  let f = fs::File::open(filename)?;
  Ok(serde_json::from_reader(f)?)
}

#[allow(dead_code)]
pub fn write_json(filename: &str, value: &Value) -> Result<()> {
  let f = fs::File::create(filename)?;
  serde_json::to_writer(f, value)?;
  Ok(())
}

#[allow(dead_code)]
pub fn download_file(url: &str, filename: PathBuf) {
  if !url.starts_with("http:") && !url.starts_with("https:") {
    fs::copy(url, filename).unwrap();
    return;
  }

  // Downloading with curl this saves us from adding
  // a Rust HTTP client dependency.
  println!("Downloading {}", url);
  let status = Command::new("curl")
    .arg("-L")
    .arg("-s")
    .arg("-o")
    .arg(&filename)
    .arg(&url)
    .status()
    .unwrap();

  assert!(status.success());
  assert!(filename.exists());
}
