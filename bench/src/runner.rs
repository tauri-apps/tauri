// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runs the benchmark apps and collects the metrics they report about themselves.

use anyhow::{Context, Result, bail};
use bench_common::{METRICS_PREFIX, SPAWN_TIME_ENV};
use std::{
  collections::BTreeMap,
  path::Path,
  process::{Child, Command, Stdio},
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tempfile::NamedTempFile;

/// What a single run of an app produced.
#[derive(Debug, Default)]
pub struct AppOutput {
  /// Time between spawning the process and it exiting.
  pub wall: Duration,
  /// The measurements the app reported.
  pub metrics: BTreeMap<String, f64>,
  /// The environment descriptions the app reported.
  pub info: BTreeMap<String, String>,
}

/// A benchmark app that was spawned and did not exit yet.
pub struct RunningApp {
  name: String,
  child: Child,
  spawned: Instant,
  // the output goes to files instead of pipes: the webview processes inherit the handles
  // and can outlive the app, which would leave a pipe reader waiting for them
  stdout: NamedTempFile,
  stderr: NamedTempFile,
}

impl RunningApp {
  pub fn spawn(exe: &Path, envs: &[(&str, &str)]) -> Result<Self> {
    let stdout = NamedTempFile::new().context("failed to create the app stdout file")?;
    let stderr = NamedTempFile::new().context("failed to create the app stderr file")?;

    let mut command = Command::new(exe);
    command
      .envs(envs.iter().copied())
      .stdin(Stdio::null())
      .stdout(stdout.reopen().context("failed to open the stdout file")?)
      .stderr(stderr.reopen().context("failed to open the stderr file")?);

    let spawn_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .context("system clock is set before the Unix epoch")?;
    command.env(SPAWN_TIME_ENV, spawn_time.as_nanos().to_string());

    let spawned = Instant::now();
    let child = command
      .spawn()
      .with_context(|| format!("failed to spawn {}", exe.display()))?;

    Ok(Self {
      name: exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned(),
      child,
      spawned,
      stdout,
      stderr,
    })
  }

  /// Waits for the app to exit by itself, and fails if it takes longer than `timeout`,
  /// exits with an error or did not report its metrics.
  pub fn wait(mut self, timeout: Duration) -> Result<AppOutput> {
    let status = loop {
      if let Some(status) = self
        .child
        .try_wait()
        .with_context(|| format!("failed to wait for {}", self.name))?
      {
        break status;
      }
      if self.spawned.elapsed() > timeout {
        self.kill();
        bail!(
          "{} did not exit within {timeout:?}\n{}",
          self.name,
          self.stderr()
        );
      }
      std::thread::sleep(Duration::from_millis(1));
    };
    let wall = self.spawned.elapsed();

    if !status.success() {
      bail!("{} exited with {status}\n{}", self.name, self.stderr());
    }

    let Some((metrics, info)) = self.reported()? else {
      bail!(
        "{} did not report its metrics\n{}",
        self.name,
        self.stderr()
      );
    };

    Ok(AppOutput {
      wall,
      metrics,
      info,
    })
  }

  /// The metrics the app printed so far, if any.
  #[allow(clippy::type_complexity)]
  fn reported(&self) -> Result<Option<(BTreeMap<String, f64>, BTreeMap<String, String>)>> {
    let stdout = std::fs::read_to_string(self.stdout.path())
      .with_context(|| format!("failed to read the output of {}", self.name))?;
    let Some(line) = stdout
      .lines()
      .find_map(|line| line.strip_prefix(METRICS_PREFIX))
    else {
      return Ok(None);
    };

    let values: BTreeMap<String, serde_json::Value> = serde_json::from_str(line)
      .with_context(|| format!("{} reported invalid metrics: {line}", self.name))?;

    let mut metrics = BTreeMap::new();
    let mut info = BTreeMap::new();
    for (name, value) in values {
      match value {
        serde_json::Value::Number(number) => {
          if let Some(number) = number.as_f64() {
            metrics.insert(name, number);
          }
        }
        serde_json::Value::String(text) => {
          info.insert(name, text);
        }
        _ => {}
      }
    }

    Ok(Some((metrics, info)))
  }

  fn stderr(&self) -> String {
    let stderr = std::fs::read_to_string(self.stderr.path()).unwrap_or_default();
    format!("stderr:\n{stderr}")
  }

  fn kill(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// Summary of the same measurement taken on several runs.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
  pub mean: f64,
  pub stddev: f64,
  pub median: f64,
  pub min: f64,
  pub max: f64,
}

impl Stats {
  /// Returns `None` when there are no values.
  pub fn new(values: &[f64]) -> Option<Self> {
    if values.is_empty() {
      return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let count = sorted.len();
    let mean = sorted.iter().sum::<f64>() / count as f64;
    let stddev = if count > 1 {
      (sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count - 1) as f64).sqrt()
    } else {
      0.
    };
    let median = if count.is_multiple_of(2) {
      (sorted[count / 2 - 1] + sorted[count / 2]) / 2.
    } else {
      sorted[count / 2]
    };

    Some(Self {
      mean,
      stddev,
      median,
      min: sorted[0],
      max: sorted[count - 1],
    })
  }
}

/// The result of running an app several times.
#[derive(Debug, Default)]
pub struct AppRuns {
  /// Wall clock time of each run, in seconds.
  pub wall: Vec<f64>,
  /// Every value reported for each metric.
  pub metrics: BTreeMap<String, Vec<f64>>,
  pub info: BTreeMap<String, String>,
}

impl AppRuns {
  /// The median of each metric: a run on a shared machine can only be slowed down by its
  /// neighbours, never sped up, so the mean is skewed by outliers.
  pub fn medians(&self) -> BTreeMap<String, f64> {
    self
      .metrics
      .iter()
      .filter_map(|(name, values)| Some((name.clone(), Stats::new(values)?.median)))
      .collect()
  }
}

/// Runs an app `warmup + runs` times and collects the results of the last `runs`.
pub fn run_app(
  exe: &Path,
  warmup: usize,
  runs: usize,
  timeout: Duration,
  pause: Duration,
) -> Result<AppRuns> {
  let mut results = AppRuns::default();

  for run in 0..warmup + runs {
    // running the apps back to back makes them slower and slower on macOS and Windows
    std::thread::sleep(pause);

    let output = RunningApp::spawn(exe, &[])?.wait(timeout)?;
    if run < warmup {
      continue;
    }

    results.wall.push(output.wall.as_secs_f64());
    for (name, value) in output.metrics {
      results.metrics.entry(name).or_default().push(value);
    }
    results.info.extend(output.info);
  }

  Ok(results)
}
