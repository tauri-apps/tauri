// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This Rust binary runs on CI and provides internal metrics results of Tauri.
//! To learn more see [benchmark_results](https://github.com/tauri-apps/benchmark_results) repository.
//!
//! ***_Internal use only_***

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

use anyhow::{Context, Result};
use std::{
  collections::{HashMap, HashSet},
  env,
  path::Path,
  process::{Command, Stdio},
  time::Instant,
};

mod utils;

/// The list of examples for benchmarks
fn get_all_benchmarks(target: &str) -> Vec<(String, String)> {
  vec![
    (
      "tauri_hello_world".into(),
      format!("../target/{target}/release/bench_helloworld"),
    ),
    (
      "tauri_cpu_intensive".into(),
      format!("../target/{target}/release/bench_cpu_intensive"),
    ),
    (
      "tauri_3mb_transfer".into(),
      format!("../target/{target}/release/bench_files_transfer"),
    ),
  ]
}

fn run_strace_benchmarks(new_data: &mut utils::BenchResult, target: &str) -> Result<()> {
  use std::io::Read;

  let mut thread_count = HashMap::<String, u64>::new();
  let mut syscall_count = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks(target) {
    let mut file = tempfile::NamedTempFile::new()
      .context("failed to create temporary file for strace output")?;

    let exe_path = utils::bench_root_path().join(&example_exe);
    let exe_path_str = exe_path
      .to_str()
      .context("executable path contains invalid UTF-8")?;
    let temp_path_str = file
      .path()
      .to_str()
      .context("temporary file path contains invalid UTF-8")?;

    Command::new("strace")
      .args(["-c", "-f", "-o", temp_path_str, exe_path_str])
      .stdout(Stdio::inherit())
      .spawn()
      .context("failed to spawn strace process")?
      .wait()
      .context("failed to wait for strace process")?;

    let mut output = String::new();
    file
      .as_file_mut()
      .read_to_string(&mut output)
      .context("failed to read strace output")?;

    let strace_result = utils::parse_strace_output(&output);
    // Count clone/clone3 syscalls as thread creation indicators
    let clone_calls = strace_result.get("clone").map(|d| d.calls).unwrap_or(0)
      + strace_result.get("clone3").map(|d| d.calls).unwrap_or(0);

    if let Some(total) = strace_result.get("total") {
      thread_count.insert(name.clone(), clone_calls);
      syscall_count.insert(name, total.calls);
    }
  }

  new_data.thread_count = thread_count;
  new_data.syscall_count = syscall_count;

  Ok(())
}

fn run_max_mem_benchmark(target: &str) -> Result<HashMap<String, u64>> {
  let mut results = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks(target) {
    let benchmark_file = utils::target_dir().join(format!("mprof{name}_.dat"));
    let benchmark_file_str = benchmark_file
      .to_str()
      .context("benchmark file path contains invalid UTF-8")?;

    let exe_path = utils::bench_root_path().join(&example_exe);
    let exe_path_str = exe_path
      .to_str()
      .context("executable path contains invalid UTF-8")?;

    let proc = Command::new("mprof")
      .args(["run", "-C", "-o", benchmark_file_str, exe_path_str])
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()
      .with_context(|| format!("failed to spawn mprof for benchmark {name}"))?;

    let proc_result = proc
      .wait_with_output()
      .with_context(|| format!("failed to wait for mprof {name}"))?;

    if !proc_result.status.success() {
      eprintln!("mprof failed for {name}: {}", String::from_utf8_lossy(&proc_result.stderr));
    }

    let mem = utils::parse_max_mem(benchmark_file_str)
      .with_context(|| format!("failed to parse mprof data for {name}"))?;
    results.insert(name, mem);

    // Clean up the temporary file
    if let Err(e) = std::fs::remove_file(&benchmark_file) {
      eprintln!("Warning: failed to remove temporary file {benchmark_file_str}: {e}");
    }
    let proc_result = proc.wait_with_output()?;
    println!("{proc_result:?}");
    if let Some(max_mem) = utils::parse_max_mem(benchmark_file)? {
      results.insert(name.to_string(), max_mem);
    }
  }

  Ok(results)
}

fn rlib_size(target_dir: &Path, prefix: &str) -> Result<u64> {
  let mut size = 0;
  let mut seen = HashSet::new();

  let deps_dir = target_dir.join("deps");
  for entry in std::fs::read_dir(&deps_dir)
    .with_context(|| format!("failed to read target deps directory: {}", deps_dir.display()))?
  {
    let entry = entry.context("failed to read directory entry")?;
    let name = entry.file_name().to_string_lossy().to_string();

    if name.starts_with(prefix) && name.ends_with(".rlib") {
      if let Some(start) = name.split('-').next() {
        if seen.insert(start.to_string()) {
          size += entry
            .metadata()
            .context("failed to read file metadata")?
            .len();
        }
      }
    }
  }

  if size == 0 {
    anyhow::bail!("no rlib files found for prefix {prefix} in {}", deps_dir.display());
  }

  Ok(size)
}

fn get_binary_sizes(target_dir: &Path, target: &str) -> Result<HashMap<String, u64>> {
  let mut sizes = HashMap::<String, u64>::new();

  let wry_size = rlib_size(target_dir, "libwry")?;
  sizes.insert("wry_rlib".to_string(), wry_size);

  for (name, example_exe) in get_all_benchmarks(target) {
    let exe_path = utils::bench_root_path().join(&example_exe);
    let meta = std::fs::metadata(&exe_path)
      .with_context(|| format!("failed to read metadata for {}", exe_path.display()))?;
    sizes.insert(name, meta.len());
  }

  Ok(sizes)
}

fn run_exec_time(target_dir: &Path, target: &str) -> Result<HashMap<String, u64>> {
  let mut exec_times = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks(target) {
    let exe_path = utils::bench_root_path().join(&example_exe);

    // Verify the executable exists
    if !exe_path.exists() {
      anyhow::bail!("Executable not found: {}", exe_path.display());
    }

    let exe_path_str = exe_path
      .to_str()
      .context("executable path contains invalid UTF-8")?;

    // Run the benchmark multiple times and take the average
    let mut total_time = 0u64;
    let runs = 3;

    for _ in 0..runs {
      let start = Instant::now();

      let status = Command::new(exe_path_str)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("failed to run benchmark {name}"))?;

      let elapsed = start.elapsed();

      if !status.success() {
        anyhow::bail!("Benchmark {name} failed with exit code: {}",
          status.code().unwrap_or(-1));
      }

      total_time += elapsed.as_millis() as u64;
    }

    let avg_time = total_time / runs as u64;
    exec_times.insert(name, avg_time);
  }

  Ok(exec_times)
}

fn cargo_deps() -> HashMap<String, String> {
  let mut deps = HashMap::<String, String>::new();

  // Get cargo metadata
  match Command::new("cargo")
    .args(["metadata", "--format-version", "1", "--no-deps"])
    .output()
  {
    Ok(output) if output.status.success() => {
      let metadata_str = String::from_utf8_lossy(&output.stdout);

      // Parse the JSON output
      if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
        if let Some(packages) = metadata["packages"].as_array() {
          for package in packages {
            if let (Some(name), Some(version)) = (
              package["name"].as_str(),
              package["version"].as_str()
            ) {
              deps.insert(name.to_string(), version.to_string());
            }
          }
        }
      }
    }
    Ok(output) => {
      eprintln!("cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Err(e) => {
      eprintln!("Failed to run cargo metadata: {}", e);
    }
  }

  // If cargo metadata failed, try to parse Cargo.lock
  if deps.is_empty() {
    if let Ok(lock_content) = std::fs::read_to_string("Cargo.lock") {
      for line in lock_content.lines() {
        if line.starts_with("name = ") {
          if let Some(name) = line.strip_prefix("name = \"").and_then(|s| s.strip_suffix("\"")) {
            // This is a simple parser - in reality you'd want a proper TOML parser
            deps.insert(name.to_string(), "unknown".to_string());
          }
        }
      }
    }
  }

  deps
}
// ... (cargo_deps and run_exec_time stay mostly the same, just add error contexts)
/// (target OS, target triple)
const TARGETS: &[(&str, &[&str])] = &[
  (
    "Windows",
    &[
      "x86_64-pc-windows-gnu",
      "i686-pc-windows-gnu",
      "i686-pc-windows-msvc",
      "x86_64-pc-windows-msvc",
    ],
  ),
  (
    "Linux",
    &[
      "x86_64-unknown-linux-gnu",
      "i686-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
    ],
  ),
  ("macOS", &["x86_64-apple-darwin", "aarch64-apple-darwin"]),
];

fn cargo_deps() -> HashMap<String, usize> {
  let mut results = HashMap::new();
  for (os, targets) in TARGETS {
    for target in *targets {
      let mut cmd = Command::new("cargo");
      cmd.arg("tree");
      cmd.arg("--no-dedupe");
      cmd.args(["--edges", "normal"]);
      cmd.args(["--prefix", "none"]);
      cmd.args(["--target", target]);
      cmd.current_dir(utils::tauri_root_path());

      let full_deps = cmd.output().expect("failed to run cargo tree").stdout;
      let full_deps = String::from_utf8(full_deps).expect("cargo tree output not utf-8");
      let count = full_deps.lines().collect::<HashSet<_>>().len() - 1; // output includes wry itself

      // set the count to the highest count seen for this OS
      let existing = results.entry(os.to_string()).or_default();
      *existing = count.max(*existing);
      assert!(count > 10); // sanity check
    }
  }
  results
}

const RESULT_KEYS: &[&str] = &["mean", "stddev", "user", "system", "min", "max"];

fn run_exec_time(target_dir: &Path) -> Result<HashMap<String, HashMap<String, f64>>> {
  let benchmark_file = target_dir.join("hyperfine_results.json");
  let benchmark_file = benchmark_file.to_str().unwrap();

  let mut command = [
    "hyperfine",
    "--export-json",
    benchmark_file,
    "--show-output",
    "--warmup",
    "3",
  ]
  .iter()
  .map(|s| s.to_string())
  .collect::<Vec<_>>();

  for (_, example_exe) in get_all_benchmarks() {
    command.push(
      utils::bench_root_path()
        .join(example_exe)
        .to_str()
        .unwrap()
        .to_string(),
    );
  }

  utils::run(&command.iter().map(|s| s.as_ref()).collect::<Vec<_>>())?;

  let mut results = HashMap::<String, HashMap<String, f64>>::new();
  let hyperfine_results = utils::read_json(benchmark_file)?;
  for ((name, _), data) in get_all_benchmarks().iter().zip(
    hyperfine_results
      .as_object()
      .unwrap()
      .get("results")
      .unwrap()
      .as_array()
      .unwrap(),
  ) {
    let data = data.as_object().unwrap().clone();
    results.insert(
      name.to_string(),
      data
        .into_iter()
        .filter(|(key, _)| RESULT_KEYS.contains(&key.as_str()))
        .map(|(key, val)| (key, val.as_f64().unwrap()))
        .collect(),
    );
  }

  Ok(results)
}

fn main() -> Result<()> {
  let json_3mb = utils::home_path().join(".tauri_3mb.json");

  if !json_3mb.exists() {
    println!("Downloading test data...");
    utils::download_file(
      "https://github.com/lemarier/tauri-test/releases/download/v2.0.0/json_3mb.json",
      json_3mb,
    ).context("failed to download test data")?;
    )?;
  }

  println!("Starting tauri benchmark");

  let target_dir = utils::target_dir();
  let target = utils::get_target();

  env::set_current_dir(utils::bench_root_path())
    .context("failed to set working directory to bench root")?;

  let format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
    .context("failed to parse time format")?;
  let now = time::OffsetDateTime::now_utc();

  println!("Running execution time benchmarks...");
  let exec_time = run_exec_time(&target_dir, &target)?;

  println!("Getting binary sizes...");
  let binary_size = get_binary_sizes(&target_dir, &target)?;

  println!("Analyzing cargo dependencies...");
  let cargo_deps = cargo_deps();

  let mut new_data = utils::BenchResult {
    created_at: now.format(&format).context("failed to format timestamp")?,
    sha1: utils::run_collect(&["git", "rev-parse", "HEAD"])
    created_at: now.format(&format).unwrap(),
    sha1: utils::run_collect(&["git", "rev-parse", "HEAD"])?
      .0
      .trim()
      .to_string(),
    exec_time,
    binary_size,
    cargo_deps,
    ..Default::default()
  };

  if cfg!(target_os = "linux") {
    println!("Running Linux-specific benchmarks...");
    run_strace_benchmarks(&mut new_data, &target)?;
    new_data.max_memory = run_max_mem_benchmark(&target)?;
  }

  println!("===== <BENCHMARK RESULTS>");
  serde_json::to_writer_pretty(std::io::stdout(), &new_data)
    .context("failed to serialize benchmark results")?;
  println!("\n===== </BENCHMARK RESULTS>");

  let bench_file = target_dir.join("bench.json");
  if let Some(filename) = bench_file.to_str() {
    utils::write_json(filename, &serde_json::to_value(&new_data)?)
      .context("failed to write benchmark results to file")?;
    println!("Results written to: {}", filename);
  } else {
    eprintln!("Cannot write bench.json, path contains invalid UTF-8");
  }

  Ok(())
}
