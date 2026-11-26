// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Utility functions for benchmarking tasks in the Tauri project.
//!
//! This module provides helpers for:
//! - Paths to project directories and targets
//! - Running and collecting process outputs
//! - Parsing syscall profiler (`strace`) outputs
//! - JSON write utilities

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::{BTreeMap, HashMap},
  fs,
  path::{Path, PathBuf},
  process::{Command, ExitStatus, Output, Stdio},
  time::{Duration, Instant},
};

/// Holds the results of a benchmark run.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct BenchResult {
  pub created_at: String,
  pub sha1: String,
  /// Wall clock time of a whole run of each app, in seconds (`mean`, `median`, `stddev`...).
  pub exec_time: HashMap<String, HashMap<String, f64>>,
  /// What each app measured about itself, as the median of all runs.
  /// Names ending in `_ms` are milliseconds, `startup_*_ms` are counted from the process spawn.
  #[serde(default)]
  pub metrics: HashMap<String, BTreeMap<String, f64>>,
  pub binary_size: HashMap<String, u64>,
  pub max_memory: HashMap<String, u64>,
  pub thread_count: HashMap<String, u64>,
  pub syscall_count: HashMap<String, u64>,
  pub cargo_deps: HashMap<String, usize>,
  /// Instructions executed by each scenario of `bench/micro`, counted by callgrind (Linux only).
  /// Unlike the timings these barely change between two runs of the same code.
  #[serde(default)]
  pub instructions: HashMap<String, u64>,
  /// Size of the installers of the hello world app, in bytes.
  #[serde(default)]
  pub bundle_size: HashMap<String, u64>,
  /// Time to build the hello world app, in seconds.
  #[serde(default)]
  pub build_time: HashMap<String, f64>,
  /// What the results were measured with (compiler, runner image, CPU...),
  /// to tell a change in Tauri apart from a change in the environment.
  #[serde(default)]
  pub environment: BTreeMap<String, String>,
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
  match (std::env::consts::OS, std::env::consts::ARCH) {
    ("macos", "aarch64") => "aarch64-apple-darwin",
    ("macos", _) => "x86_64-apple-darwin",
    ("windows", "aarch64") => "aarch64-pc-windows-msvc",
    ("windows", _) => "x86_64-pc-windows-msvc",
    (_, "aarch64") => "aarch64-unknown-linux-gnu",
    _ => "x86_64-unknown-linux-gnu",
  }
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

/// Spawn a command and wait for it to exit, killing it when it runs for longer than `timeout`.
///
/// A benchmark app only exits when its frontend reports back through the IPC,
/// so without a timeout a broken app hangs the whole job.
pub fn wait_with_timeout(command: &mut Command, timeout: Duration) -> Result<ExitStatus> {
  let mut child = command
    .spawn()
    .with_context(|| format!("failed to spawn {command:?}"))?;
  let start = Instant::now();

  loop {
    if let Some(status) = child
      .try_wait()
      .with_context(|| format!("failed to wait for {command:?}"))?
    {
      return Ok(status);
    }

    if start.elapsed() > timeout {
      let _ = child.kill();
      let _ = child.wait();
      bail!("{command:?} did not exit within {timeout:?}");
    }

    std::thread::sleep(Duration::from_millis(20));
  }
}

/// Write a [`serde_json::Value`] into a JSON file.
pub fn write_json(filename: &Path, value: &Value) -> Result<()> {
  let f = fs::File::create(filename)
    .with_context(|| format!("failed to create JSON file {}", filename.display()))?;
  serde_json::to_writer(f, value)?;
  Ok(())
}
