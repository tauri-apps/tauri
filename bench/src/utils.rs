// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Utility functions for benchmarking tasks in the Tauri project.
//!
//! This module provides helpers for:
//! - Paths to project directories and targets
//! - Running and collecting process outputs
//! - Parsing memory profiler (`mprof`) and syscall profiler (`strace`) outputs
//! - JSON read/write utilities
//! - File download utilities (via `curl` or file copy)

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::HashMap,
  fs,
  io::{BufRead, BufReader},
  path::PathBuf,
  process::{Command, Output, Stdio},
};

/// Holds the results of a benchmark run.
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

/// Represents a single line of parsed `strace` output.
#[derive(Debug, Clone, Serialize)]
pub struct StraceOutput {
  pub percent_time: f64,
  pub seconds: f64,
  pub usecs_per_call: Option<u64>,
  pub calls: u64,
  pub errors: u64,
}

/// Get the compilation target triple for the current platform.
pub fn get_target() -> &'static str {
  #[cfg(target_os = "macos")]
  return if cfg!(target_arch = "aarch64") {
    "aarch64-apple-darwin"
  } else {
    "x86_64-apple-darwin"
  };

  #[cfg(target_os = "ios")]
  return if cfg!(target_arch = "aarch64") {
    "aarch64-apple-ios"
  } else {
    "x86_64-apple-ios"
  };

  #[cfg(target_os = "linux")]
  return "x86_64-unknown-linux-gnu";

  #[cfg(target_os = "windows")]
  unimplemented!("Windows target not implemented yet");
}

/// Get the `target/release` directory path for benchmarks.
pub fn target_dir() -> PathBuf {
  bench_root_path()
    .join("..")
    .join("target")
    .join(get_target())
    .join("release")
}

/// Get the root path of the current benchmark crate.
pub fn bench_root_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Get the home directory of the current user.
pub fn home_path() -> PathBuf {
  #[cfg(any(target_os = "macos", target_os = "ios", target_os = "linux"))]
  {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
  }

  #[cfg(target_os = "windows")]
  {
    PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default())
  }
}

/// Get the root path of the Tauri repository.
pub fn tauri_root_path() -> PathBuf {
  bench_root_path().parent().map(|p| p.to_path_buf()).unwrap()
}

/// Run a command and collect its stdout and stderr as strings.
/// Returns an error if the command fails or exits with a non-zero status.
pub fn run_collect(cmd: &[&str]) -> Result<(String, String)> {
  let output: Output = Command::new(cmd[0])
    .args(&cmd[1..])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .with_context(|| format!("failed to execute command: {cmd:?}"))?;

  if !output.status.success() {
    bail!(
      "Command {:?} exited with {:?}\nstdout:\n{}\nstderr:\n{}",
      cmd,
      output.status.code(),
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr)
    );
  }

  Ok((
    String::from_utf8_lossy(&output.stdout).to_string(),
    String::from_utf8_lossy(&output.stderr).to_string(),
  ))
}

/// Parse a memory profiler (`mprof`) output file and return the maximum
/// memory usage in bytes. Returns `None` if no values are found.
pub fn parse_max_mem(file_path: &str) -> Result<Option<u64>> {
  let file = fs::File::open(file_path)
    .with_context(|| format!("failed to open mprof output file {file_path}"))?;
  let output = BufReader::new(file);

  let mut highest: u64 = 0;

  for line in output.lines().map_while(Result::ok) {
    let split: Vec<&str> = line.split(' ').collect();
    if split.len() == 3 {
      if let Ok(mb) = split[1].parse::<f64>() {
        let current_bytes = (mb * 1024.0 * 1024.0) as u64;
        highest = highest.max(current_bytes);
      }
    }
  }

  // Best-effort cleanup
  let _ = fs::remove_file(file_path);

  Ok(if highest > 0 { Some(highest) } else { None })
}

/// Parse the output of `strace -c` and return a summary of syscalls.
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
    let syscall_fields: Vec<&str> = line.split_whitespace().collect();
    let len = syscall_fields.len();

    if let Some(&syscall_name) = syscall_fields.last() {
      if (5..=6).contains(&len) {
        let output = StraceOutput {
          percent_time: syscall_fields[0].parse().unwrap_or(0.0),
          seconds: syscall_fields[1].parse().unwrap_or(0.0),
          usecs_per_call: syscall_fields[2].parse().ok(),
          calls: syscall_fields[3].parse().unwrap_or(0),
          errors: if len < 6 {
            0
          } else {
            syscall_fields[4].parse().unwrap_or(0)
          },
        };
        summary.insert(syscall_name.to_string(), output);
      }
    }
  }

  let total_fields: Vec<&str> = total_line.split_whitespace().collect();
  let total = match total_fields.len() {
    5 => StraceOutput {
      percent_time: total_fields[0].parse().unwrap_or(0.0),
      seconds: total_fields[1].parse().unwrap_or(0.0),
      usecs_per_call: None,
      calls: total_fields[2].parse().unwrap_or(0),
      errors: total_fields[3].parse().unwrap_or(0),
    },
    6 => StraceOutput {
      percent_time: total_fields[0].parse().unwrap_or(0.0),
      seconds: total_fields[1].parse().unwrap_or(0.0),
      usecs_per_call: total_fields[2].parse().ok(),
      calls: total_fields[3].parse().unwrap_or(0),
      errors: total_fields[4].parse().unwrap_or(0),
    },
    _ => {
      panic!("Unexpected total field count: {}", total_fields.len());
    }
  };

  summary.insert("total".to_string(), total);
  summary
}

/// Run a command and wait for completion.
/// Returns an error if the command fails.
pub fn run(cmd: &[&str]) -> Result<()> {
  let status = Command::new(cmd[0])
    .args(&cmd[1..])
    .stdin(Stdio::piped())
    .status()
    .with_context(|| format!("failed to execute command: {cmd:?}"))?;

  if !status.success() {
    bail!("Command {:?} exited with {:?}", cmd, status.code());
  }
  Ok(())
}

/// Read a JSON file into a [`serde_json::Value`].
pub fn read_json(filename: &str) -> Result<Value> {
  let f =
    fs::File::open(filename).with_context(|| format!("failed to open JSON file {filename}"))?;
  Ok(serde_json::from_reader(f)?)
}

/// Write a [`serde_json::Value`] into a JSON file.
pub fn write_json(filename: &str, value: &Value) -> Result<()> {
  let f =
    fs::File::create(filename).with_context(|| format!("failed to create JSON file {filename}"))?;
  serde_json::to_writer(f, value)?;
  Ok(())
}

/// Download a file from either a local path or an HTTP/HTTPS URL.
/// Falls back to copying the file if the URL does not start with http/https.
pub fn download_file(url: &str, filename: PathBuf) -> Result<()> {
  if !url.starts_with("http:") && !url.starts_with("https:") {
    fs::copy(url, &filename).with_context(|| format!("failed to copy from {url}"))?;
    return Ok(());
  }

  println!("Downloading {url}");
  let status = Command::new("curl")
    .arg("-L")
    .arg("-s")
    .arg("-o")
    .arg(&filename)
    .arg(url)
    .status()
    .with_context(|| format!("failed to execute curl for {url}"))?;

  if !status.success() {
    bail!("curl failed with exit code {:?}", status.code());
  }
  if !filename.exists() {
    bail!("expected file {:?} to exist after download", filename);
  }

  Ok(())
}
