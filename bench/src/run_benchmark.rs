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
  collections::{BTreeMap, HashMap, HashSet},
  env,
  path::Path,
  process::{Command, Stdio},
  time::Duration,
};
use time::format_description::well_known::Rfc3339;

mod process;
mod runner;
mod utils;

/// How long a single run of a benchmark app may take.
const APP_TIMEOUT: Duration = Duration::from_secs(60);
/// Time to wait between two runs.
const RUN_PAUSE: Duration = Duration::from_secs(1);
/// How many times the resource usage of an app is measured.
const USAGE_RUNS: usize = 3;
/// Time an app is left alone before its idle resource usage is measured.
const IDLE_SETTLE: Duration = Duration::from_secs(3);
/// For how long the CPU usage of an idle app is observed.
const IDLE_WINDOW: Duration = Duration::from_secs(5);
/// How many extra windows the hello world app opens to measure the cost of a window.
const EXTRA_WINDOWS: usize = 5;

struct Benchmark {
  name: &'static str,
  /// Path of the executable, relative to the bench directory.
  exe: String,
  /// Runs that are not measured.
  warmup: usize,
  /// Runs that are measured.
  runs: usize,
  /// Whether to also measure the app sitting idle and the cost of opening more windows.
  idle: bool,
}

/// The list of examples for benchmarks
fn get_all_benchmarks(target: &str) -> Vec<Benchmark> {
  let exe = |name: &str| {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    format!("../target/{target}/release/{name}{extension}")
  };
  vec![
    Benchmark {
      name: "tauri_hello_world",
      exe: exe("bench_helloworld"),
      warmup: 3,
      runs: 20,
      idle: true,
    },
    // Not a benchmark of Tauri: nearly all of its time is spent by the JavaScript engine
    // finding prime numbers. It tells how fast the machine the benchmarks ran on is,
    // which the other timings can be compared against.
    Benchmark {
      name: "tauri_cpu_intensive",
      exe: exe("bench_cpu_intensive"),
      warmup: 1,
      runs: 5,
      idle: false,
    },
    // hello world with the frontend of a real application
    Benchmark {
      name: "tauri_assets",
      exe: exe("bench_assets"),
      warmup: 3,
      runs: 10,
      idle: false,
    },
    // times the IPC from the inside, so it needs fewer runs
    Benchmark {
      name: "tauri_ipc",
      exe: exe("bench_ipc"),
      warmup: 1,
      runs: 5,
      idle: false,
    },
  ]
}

fn run_strace_benchmarks(new_data: &mut utils::BenchResult, target: &str) -> Result<()> {
  use std::io::Read;

  let mut thread_count = HashMap::<String, u64>::new();
  let mut syscall_count = HashMap::<String, u64>::new();

  for Benchmark {
    name,
    exe: example_exe,
    ..
  } in get_all_benchmarks(target)
  {
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

    // strace exits with the status of the traced app
    let status = utils::wait_with_timeout(
      Command::new("strace")
        .args(["-c", "-f", "-o", temp_path_str, exe_path_str])
        .stdout(Stdio::inherit()),
      APP_TIMEOUT,
    )
    .with_context(|| format!("failed to run strace for benchmark {name}"))?;
    if !status.success() {
      anyhow::bail!("strace for benchmark {name} exited with {status}");
    }

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
      thread_count.insert(name.to_string(), clone_calls);
      syscall_count.insert(name.to_string(), total.calls);
    }
  }

  new_data.thread_count = thread_count;
  new_data.syscall_count = syscall_count;

  Ok(())
}

fn median(values: &[f64]) -> f64 {
  runner::Stats::new(values)
    .map(|stats| stats.median)
    .unwrap_or_default()
}

/// Starts an app that stays alive once it is done, and waits for it to get there.
fn hold_app(exe: &Path, envs: &[(&str, &str)]) -> Result<(runner::RunningApp, runner::AppOutput)> {
  let mut envs = envs.to_vec();
  envs.push((bench_common::HOLD_ENV, "1"));
  let mut app = runner::RunningApp::spawn(exe, &envs)?;
  let output = app.wait_for_report(APP_TIMEOUT)?;
  Ok((app, output))
}

/// Measures what the apps use while they are running, with the processes of the webview included.
///
/// The apps are asked to stay alive after their workload, so the usage is read at a known point
/// instead of being sampled from an app that exits as soon as it is done:
/// - `max_memory`: sum of the peak resident memory of each process (Linux)
/// - `memory_bytes`: memory in use once the workload is done
/// - `idle_*`: memory, CPU time and wakeups of an app that is doing nothing
/// - `open_window_ms`, `window_memory_bytes`: cost of each additional window
fn run_usage_benchmarks(
  target: &str,
  metrics: &mut HashMap<String, BTreeMap<String, f64>>,
) -> Result<HashMap<String, u64>> {
  let mut max_memory = HashMap::new();

  for benchmark in get_all_benchmarks(target) {
    let name = benchmark.name;
    println!("  {name}");
    let exe_path = utils::bench_root_path().join(&benchmark.exe);
    let mut samples = BTreeMap::<&str, Vec<f64>>::new();
    let mut sample = |name, value| samples.entry(name).or_default().push(value);

    for _ in 0..USAGE_RUNS {
      std::thread::sleep(RUN_PAUSE);

      if process::SUPPORTED {
        let (app, _) =
          hold_app(&exe_path, &[]).with_context(|| format!("failed to run benchmark {name}"))?;
        let done = process::usage(app.pid())?;
        sample("memory_bytes", done.memory as f64);
        if done.peak_memory > 0 {
          sample("peak_memory_bytes", done.peak_memory as f64);
        }

        if benchmark.idle {
          std::thread::sleep(IDLE_SETTLE);
          let before = process::usage(app.pid())?;
          std::thread::sleep(IDLE_WINDOW);
          let after = process::usage(app.pid())?;
          let seconds = IDLE_WINDOW.as_secs_f64();

          sample("idle_memory_bytes", after.memory as f64);
          sample("idle_app_memory_bytes", after.app_memory as f64);
          sample(
            "idle_cpu_ms_per_s",
            (after.cpu_ms - before.cpu_ms) / seconds,
          );
          sample(
            "idle_app_cpu_ms_per_s",
            (after.app_cpu_ms - before.app_cpu_ms) / seconds,
          );
          if cfg!(target_os = "linux") {
            sample(
              "idle_app_wakeups_per_s",
              after.app_wakeups.saturating_sub(before.app_wakeups) as f64 / seconds,
            );
            sample("idle_processes", after.processes as f64);
            sample("idle_threads", after.threads as f64);
          }
        }
      }

      if benchmark.idle {
        std::thread::sleep(RUN_PAUSE);

        let windows = EXTRA_WINDOWS.to_string();
        let (app, output) = hold_app(&exe_path, &[(bench_common::WINDOWS_ENV, &windows)])
          .with_context(|| format!("failed to run benchmark {name} with {windows} windows"))?;
        if let Some(open_window_ms) = output.metrics.get("open_window_ms") {
          sample("open_window_ms", *open_window_ms);
        }

        if process::SUPPORTED {
          std::thread::sleep(IDLE_SETTLE);
          sample(
            "windows_memory_bytes",
            process::usage(app.pid())?.memory as f64,
          );
        }
      }
    }

    let mut medians: BTreeMap<String, f64> = samples
      .iter()
      .map(|(name, values)| (name.to_string(), median(values)))
      .collect();

    // the history of this value is published under its own key
    if let Some(peak) = medians.remove("peak_memory_bytes") {
      max_memory.insert(name.to_string(), peak as u64);
    }
    // what each of the extra windows added to the idle app
    if let Some(windows_memory) = medians.remove("windows_memory_bytes")
      && let Some(idle_memory) = medians.get("idle_memory_bytes")
    {
      medians.insert(
        "window_memory_bytes".into(),
        ((windows_memory - idle_memory) / EXTRA_WINDOWS as f64).max(0.),
      );
    }

    metrics.entry(name.to_string()).or_default().extend(medians);
  }

  Ok(max_memory)
}

/// Size of the rlib of a workspace dependency.
fn rlib_size(target_dir: &std::path::Path, library: &str) -> Result<u64> {
  let prefix = format!("lib{library}-");

  // Cargo's new build directory layout keeps every crate in a directory of its own
  // (build/<crate>/<hash>/out), the previous one puts everything in deps/
  let build_dir = target_dir.join("build").join(library);
  let mut candidates = Vec::new();
  if build_dir.is_dir() {
    for entry in std::fs::read_dir(&build_dir)
      .with_context(|| format!("failed to read {}", build_dir.display()))?
    {
      candidates.push(
        entry
          .context("failed to read directory entry")?
          .path()
          .join("out"),
      );
    }
  }
  candidates.push(target_dir.join("deps"));

  // the crate can be built more than once (e.g. also as a build dependency), take the newest
  let mut newest: Option<(std::time::SystemTime, u64)> = None;
  for dir in candidates {
    let Ok(files) = std::fs::read_dir(&dir) else {
      continue;
    };
    for file in files {
      let file = file.context("failed to read directory entry")?;
      let name = file.file_name().to_string_lossy().to_string();
      if name.starts_with(&prefix) && name.ends_with(".rlib") {
        let metadata = file.metadata().context("failed to read file metadata")?;
        let modified = metadata.modified().context("failed to read file mtime")?;
        if newest.is_none_or(|(time, _)| modified > time) {
          newest = Some((modified, metadata.len()));
        }
      }
    }
  }

  newest
    .map(|(_, size)| size)
    .with_context(|| format!("no {prefix}*.rlib file found in {}", target_dir.display()))
}

fn get_binary_sizes(target_dir: &Path, target: &str) -> Result<HashMap<String, u64>> {
  let mut sizes = HashMap::<String, u64>::new();

  let wry_size = rlib_size(target_dir, "wry")?;
  sizes.insert("wry_rlib".to_string(), wry_size);

  for Benchmark {
    name,
    exe: example_exe,
    ..
  } in get_all_benchmarks(target)
  {
    let exe_path = utils::bench_root_path().join(&example_exe);
    let meta = std::fs::metadata(&exe_path)
      .with_context(|| format!("failed to read metadata for {}", exe_path.display()))?;
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
      // without `-p` cargo walks every workspace member (CLI, bundler, examples...)
      cmd.args(["-p", "tauri"]);
      cmd.arg("--no-dedupe");
      cmd.args(["--edges", "normal"]);
      cmd.args(["--prefix", "none"]);
      cmd.args(["--target", target]);
      cmd.current_dir(utils::tauri_root_path());

      match cmd.output() {
        Ok(output) if output.status.success() => {
          let full_deps = String::from_utf8_lossy(&output.stdout);
          let count = full_deps
            .lines()
            .collect::<HashSet<_>>()
            .len()
            .saturating_sub(1); // output includes tauri itself

          // set the count to the highest count seen for this OS
          let existing = results.entry(os.to_string()).or_default();
          *existing = count.max(*existing);

          if count <= 10 {
            eprintln!("Warning: dependency count for {target} seems low: {count}");
          }
        }
        Ok(output) => {
          eprintln!(
            "cargo tree failed for {target}: {}",
            String::from_utf8_lossy(&output.stderr)
          );
        }
        Err(e) => {
          eprintln!("Failed to run cargo tree for {target}: {e}");
        }
      }
    }
  }
  results
}

/// Describes the toolchain and the machine the benchmark runs on.
fn environment() -> BTreeMap<String, String> {
  let mut environment = BTreeMap::new();

  // `cargo run` exports RUSTUP_TOOLCHAIN, so this is the toolchain running the benchmark,
  // which on CI is the one that built the apps
  match utils::run_collect(&["rustc", "--version"]) {
    Ok((stdout, _)) => {
      environment.insert("rustc".into(), stdout.trim().into());
    }
    Err(e) => eprintln!("failed to read the rustc version: {e:#}"),
  }

  environment.insert("arch".into(), env::consts::ARCH.into());
  if let Ok(cpus) = std::thread::available_parallelism() {
    environment.insert("cpus".into(), cpus.to_string());
  }
  #[cfg(target_os = "linux")]
  if let Some(model) = std::fs::read_to_string("/proc/cpuinfo")
    .ok()
    .and_then(|cpuinfo| {
      cpuinfo
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split(':').nth(1))
        .map(|model| model.trim().to_string())
    })
  {
    environment.insert("cpu".into(), model);
  }

  // set on GitHub hosted runners
  for (key, var) in [
    ("runner_image", "ImageOS"),
    ("runner_image_version", "ImageVersion"),
  ] {
    if let Ok(value) = env::var(var) {
      environment.insert(key.into(), value);
    }
  }

  environment
}

/// Runs every app several times, returning the wall clock time of the runs
/// and the metrics the apps reported about themselves.
#[allow(clippy::type_complexity)]
fn run_apps(
  target: &str,
  environment: &mut BTreeMap<String, String>,
) -> Result<(
  HashMap<String, HashMap<String, f64>>,
  HashMap<String, BTreeMap<String, f64>>,
)> {
  let mut exec_time = HashMap::new();
  let mut metrics = HashMap::new();

  for benchmark in get_all_benchmarks(target) {
    let name = benchmark.name;
    println!("  {name}");
    let exe_path = utils::bench_root_path().join(&benchmark.exe);
    let runs = runner::run_app(
      &exe_path,
      benchmark.warmup,
      benchmark.runs,
      APP_TIMEOUT,
      RUN_PAUSE,
    )
    .with_context(|| format!("failed to run benchmark {name}"))?;

    let wall = runner::Stats::new(&runs.wall).context("benchmark did not run")?;
    exec_time.insert(
      name.to_string(),
      HashMap::from([
        ("mean".to_string(), wall.mean),
        ("stddev".to_string(), wall.stddev),
        ("median".to_string(), wall.median),
        ("min".to_string(), wall.min),
        ("max".to_string(), wall.max),
      ]),
    );
    metrics.insert(name.to_string(), runs.medians());

    if let Some(version) = runs.info.get("webview_version") {
      environment.insert("webview".into(), version.clone());
    }
  }

  Ok((exec_time, metrics))
}

fn main() -> Result<()> {
  println!("Starting tauri benchmark");

  let target_dir = utils::target_dir();
  let target = utils::get_target();

  env::set_current_dir(utils::bench_root_path())
    .context("failed to set working directory to bench root")?;

  let mut environment = environment();

  println!("Running the apps...");
  let (exec_time, metrics) = run_apps(target, &mut environment)?;

  println!("Getting binary sizes...");
  let binary_size = get_binary_sizes(&target_dir, target)?;

  println!("Analyzing cargo dependencies...");
  let cargo_deps = cargo_deps();

  let mut new_data = utils::BenchResult {
    created_at: time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
    sha1: {
      let output = utils::run_collect(&["git", "rev-parse", "HEAD"])?;
      output.0.trim().to_string()
    },
    exec_time,
    metrics,
    binary_size,
    cargo_deps,
    environment,
    ..Default::default()
  };

  if cfg!(target_os = "linux") {
    println!("Running Linux-specific benchmarks...");
    run_strace_benchmarks(&mut new_data, target)?;
  }

  println!("Measuring resource usage...");
  new_data.max_memory = run_usage_benchmarks(target, &mut new_data.metrics)?;

  println!("===== <BENCHMARK RESULTS>");
  serde_json::to_writer_pretty(std::io::stdout(), &new_data)
    .context("failed to serialize benchmark results")?;
  println!("\n===== </BENCHMARK RESULTS>");

  let bench_file = target_dir.join("bench.json");
  utils::write_json(&bench_file, &serde_json::to_value(&new_data)?)
    .context("failed to write benchmark results to file")?;
  println!("Results written to: {}", bench_file.display());

  Ok(())
}
