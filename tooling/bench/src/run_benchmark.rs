// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::{
  collections::{HashMap, HashSet},
  env,
  path::Path,
  process::{Command, Stdio},
};

mod utils;

/// The list of the examples of the benchmark name and binary relative path
fn get_all_benchmarks() -> Vec<(String, String)> {
  vec![
    (
      "tauri_hello_world".into(),
      format!(
        "tests/target/{}/release/bench_helloworld",
        utils::get_target()
      ),
    ),
    (
      "tauri_cpu_intensive".into(),
      format!(
        "tests/target/{}/release/bench_cpu_intensive",
        utils::get_target()
      ),
    ),
    (
      "tauri_3mb_transfer".into(),
      format!(
        "tests/target/{}/release/bench_files_transfer",
        utils::get_target()
      ),
    ),
  ]
}

fn run_strace_benchmarks(new_data: &mut utils::BenchResult) -> Result<()> {
  use std::io::Read;

  let mut thread_count = HashMap::<String, u64>::new();
  let mut syscall_count = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks() {
    let mut file = tempfile::NamedTempFile::new()?;

    Command::new("strace")
      .args(&[
        "-c",
        "-f",
        "-o",
        file.path().to_str().unwrap(),
        utils::bench_root_path().join(example_exe).to_str().unwrap(),
      ])
      .stdout(Stdio::inherit())
      .spawn()?
      .wait()?;

    let mut output = String::new();
    file.as_file_mut().read_to_string(&mut output)?;

    let strace_result = utils::parse_strace_output(&output);
    let clone = strace_result.get("clone").map(|d| d.calls).unwrap_or(0) + 1;
    let total = strace_result.get("total").unwrap().calls;
    thread_count.insert(name.to_string(), clone);
    syscall_count.insert(name.to_string(), total);
  }

  new_data.thread_count = thread_count;
  new_data.syscall_count = syscall_count;

  Ok(())
}

fn run_max_mem_benchmark() -> Result<HashMap<String, u64>> {
  let mut results = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks() {
    let benchmark_file = utils::target_dir().join(format!("mprof{}_.dat", name));
    let benchmark_file = benchmark_file.to_str().unwrap();

    let proc = Command::new("mprof")
      .args(&[
        "run",
        "-C",
        "-o",
        benchmark_file,
        utils::bench_root_path().join(example_exe).to_str().unwrap(),
      ])
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()?;

    let proc_result = proc.wait_with_output()?;
    println!("{:?}", proc_result);
    results.insert(
      name.to_string(),
      utils::parse_max_mem(&benchmark_file).unwrap(),
    );
  }

  Ok(results)
}

fn rlib_size(target_dir: &std::path::Path, prefix: &str) -> u64 {
  let mut size = 0;
  let mut seen = std::collections::HashSet::new();

  for entry in std::fs::read_dir(target_dir.join("deps")).unwrap() {
    let entry = entry.unwrap();
    let os_str = entry.file_name();
    let name = os_str.to_str().unwrap();
    if name.starts_with(prefix) && name.ends_with(".rlib") {
      let start = name.split('-').next().unwrap().to_string();
      if seen.contains(&start) {
        println!("skip {}", name);
      } else {
        seen.insert(start);
        size += entry.metadata().unwrap().len();
        println!("check size {} {}", name, size);
      }
    }
  }
  assert!(size > 0);
  size
}

fn get_binary_sizes(target_dir: &Path) -> Result<HashMap<String, u64>> {
  let mut sizes = HashMap::<String, u64>::new();

  let wry_size = rlib_size(&target_dir, "libwry");
  println!("wry {} bytes", wry_size);
  sizes.insert("wry_rlib".to_string(), wry_size);

  // add size for all EXEC_TIME_BENCHMARKS
  for (name, example_exe) in get_all_benchmarks() {
    let meta = std::fs::metadata(example_exe).unwrap();
    sizes.insert(name.to_string(), meta.len());
  }

  Ok(sizes)
}

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
      cmd.args(&["--edges", "normal"]);
      cmd.args(&["--prefix", "none"]);
      cmd.args(&["--target", target]);
      cmd.current_dir(&utils::tauri_root_path());

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

  utils::run(&command.iter().map(|s| s.as_ref()).collect::<Vec<_>>());

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
  // download big files if not present
  let json_3mb = utils::home_path().join(".tauri_3mb.json");

  if !json_3mb.exists() {
    utils::download_file(
      "https://github.com/lemarier/tauri-test/releases/download/v2.0.0/json_3mb.json",
      json_3mb,
    );
  }

  println!("Starting tauri benchmark");

  let target_dir = utils::target_dir();

  env::set_current_dir(&utils::bench_root_path())?;

  let mut new_data = utils::BenchResult {
    created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    sha1: utils::run_collect(&["git", "rev-parse", "HEAD"])
      .0
      .trim()
      .to_string(),
    exec_time: run_exec_time(&target_dir)?,
    binary_size: get_binary_sizes(&target_dir)?,
    cargo_deps: cargo_deps(),
    ..Default::default()
  };

  if cfg!(target_os = "linux") {
    run_strace_benchmarks(&mut new_data)?;
    new_data.max_memory = run_max_mem_benchmark()?;
  }

  println!("===== <BENCHMARK RESULTS>");
  serde_json::to_writer_pretty(std::io::stdout(), &new_data)?;
  println!("\n===== </BENCHMARK RESULTS>");

  if let Some(filename) = target_dir.join("bench.json").to_str() {
    utils::write_json(filename, &serde_json::to_value(&new_data)?)?;
  } else {
    eprintln!("Cannot write bench.json, path is invalid");
  }

  Ok(())
}
