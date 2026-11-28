// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Resource usage of a running app, including the processes of its webview.

use anyhow::Result;

/// Resource usage of a process tree at a point in time.
#[derive(Debug, Default, Clone, Copy)]
pub struct Usage {
  /// Memory in use, in bytes.
  ///
  /// On Linux this is the proportional set size: pages shared between processes (the webview
  /// libraries, mostly) are split between them instead of being counted once per process.
  /// On Windows it is the sum of the working sets, which does count shared pages more than once.
  pub memory: u64,
  /// Sum of the highest resident set size each process reached, in bytes. Linux only.
  pub peak_memory: u64,
  /// Memory in use by the app process alone, without the webview processes.
  pub app_memory: u64,
  /// CPU time used so far by the whole tree, in milliseconds.
  pub cpu_ms: f64,
  /// CPU time used so far by the app process alone, in milliseconds.
  pub app_cpu_ms: f64,
  /// How many times the threads of the app process went to sleep waiting for something
  /// (voluntary context switches), which is how many times they woke up. Linux only.
  pub app_wakeups: u64,
  pub processes: usize,
  pub threads: usize,
}

/// Measures the app with the given process ID and everything it spawned.
pub fn usage(pid: u32) -> Result<Usage> {
  imp::usage(pid)
}

/// Kills whatever is left of the process tree of an app.
pub fn kill_tree(pid: u32) {
  imp::kill_tree(pid)
}

/// Whether [`usage`] is implemented for this platform.
pub const SUPPORTED: bool = cfg!(any(target_os = "linux", windows));

#[cfg(target_os = "linux")]
mod imp {
  use super::Usage;
  use anyhow::{Context, Result};
  use std::{
    collections::{HashMap, HashSet},
    fs,
    process::Command,
  };

  /// Clock ticks per second of the times in /proc/<pid>/stat.
  /// This is `sysconf(_SC_CLK_TCK)`, which is 100 on every Linux architecture.
  const CLOCK_TICKS: f64 = 100.;

  fn pids() -> impl Iterator<Item = u32> {
    fs::read_dir("/proc")
      .into_iter()
      .flatten()
      .flatten()
      .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
  }

  /// The fields of /proc/<pid>/stat after the process name,
  /// which is skipped because it can contain spaces and parentheses.
  fn stat(pid: u32) -> Option<Vec<String>> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_, fields) = stat.rsplit_once(')')?;
    Some(fields.split_whitespace().map(Into::into).collect())
  }

  /// The app and all its descendants.
  ///
  /// The webview processes can be started through a sandbox helper that exits right away,
  /// which re-parents them to init, so the tree is tracked by process group as well:
  /// the runner starts each app in a group of its own.
  fn tree(root: u32) -> Vec<u32> {
    let mut children = HashMap::<u32, Vec<u32>>::new();
    let mut groups = HashMap::<u32, u32>::new();
    for pid in pids() {
      let Some(fields) = stat(pid) else { continue };
      // fields[0] is the state, the third field of the file
      if let (Some(parent), Some(group)) = (
        fields.get(1).and_then(|f| f.parse().ok()),
        fields.get(2).and_then(|f| f.parse::<u32>().ok()),
      ) {
        children.entry(parent).or_default().push(pid);
        groups.insert(pid, group);
      }
    }

    let mut tree = HashSet::from([root]);
    let mut pending = vec![root];
    while let Some(pid) = pending.pop() {
      for child in children.get(&pid).into_iter().flatten() {
        if tree.insert(*child) {
          pending.push(*child);
        }
      }
    }

    // only when the app leads its group, otherwise this would be the group of the runner
    if groups.get(&root) == Some(&root) {
      tree.extend(
        groups
          .iter()
          .filter(|(_, group)| **group == root)
          .map(|(pid, _)| *pid),
      );
    }

    tree.into_iter().collect()
  }

  /// Reads a `Name:   123 kB` line of a /proc status-like file, in bytes.
  fn kilobytes(content: &str, name: &str) -> Option<u64> {
    content
      .lines()
      .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))
      .and_then(|value| value.trim().strip_suffix("kB")?.trim().parse::<u64>().ok())
      .map(|kb| kb * 1024)
  }

  fn count(content: &str, name: &str) -> Option<u64> {
    content
      .lines()
      .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))
      .and_then(|value| value.trim().parse().ok())
  }

  pub fn usage(root: u32) -> Result<Usage> {
    let mut usage = Usage::default();

    for pid in tree(root) {
      // a process can exit at any point while it is being read
      let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) else {
        continue;
      };
      let Some(stat) = stat(pid) else { continue };

      let memory = fs::read_to_string(format!("/proc/{pid}/smaps_rollup"))
        .ok()
        .and_then(|rollup| kilobytes(&rollup, "Pss"))
        // kernel threads and zombies have no memory
        .unwrap_or_default();
      // utime and stime are the 14th and 15th fields of the file
      let cpu_ms = [11, 12]
        .iter()
        .filter_map(|index| stat.get(*index)?.parse::<f64>().ok())
        .sum::<f64>()
        / CLOCK_TICKS
        * 1000.;

      usage.memory += memory;
      usage.peak_memory += kilobytes(&status, "VmHWM").unwrap_or_default();
      usage.cpu_ms += cpu_ms;
      usage.processes += 1;
      usage.threads += count(&status, "Threads").unwrap_or_default() as usize;

      if pid == root {
        usage.app_memory = memory;
        usage.app_cpu_ms = cpu_ms;
        for task in fs::read_dir(format!("/proc/{pid}/task"))
          .into_iter()
          .flatten()
          .flatten()
        {
          if let Ok(status) = fs::read_to_string(task.path().join("status")) {
            usage.app_wakeups += count(&status, "voluntary_ctxt_switches").unwrap_or_default();
          }
        }
      }
    }

    if usage.processes == 0 {
      anyhow::bail!("process {root} is not running");
    }
    fs::metadata(format!("/proc/{root}")).with_context(|| format!("process {root} exited"))?;

    Ok(usage)
  }

  pub fn kill_tree(root: u32) {
    let pids = tree(root);
    let _ = Command::new("kill")
      .arg("-KILL")
      .args(pids.iter().map(u32::to_string))
      .stderr(std::process::Stdio::null())
      .status();
  }
}

#[cfg(windows)]
mod imp {
  use super::Usage;
  use anyhow::{Context, Result};
  use std::collections::HashSet;
  use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

  fn tree(system: &System, root: u32) -> HashSet<Pid> {
    let mut tree = HashSet::from([Pid::from_u32(root)]);
    // parents are not listed before their children, so iterate until nothing is added
    loop {
      let mut added = false;
      for (pid, process) in system.processes() {
        if let Some(parent) = process.parent() {
          if tree.contains(&parent) && tree.insert(*pid) {
            added = true;
          }
        }
      }
      if !added {
        break tree;
      }
    }
  }

  fn system() -> Result<System> {
    let mut system = System::new().context("failed to query the system processes")?;
    system.refresh_processes_specifics(
      ProcessesToUpdate::All,
      true,
      ProcessRefreshKind::nothing().with_memory().with_cpu(),
    );
    Ok(system)
  }

  pub fn usage(root: u32) -> Result<Usage> {
    let system = system()?;
    let mut usage = Usage::default();

    for pid in tree(&system, root) {
      let Some(process) = system.process(pid) else {
        continue;
      };
      let cpu_ms = process.accumulated_cpu_time() as f64;
      usage.memory += process.memory();
      usage.cpu_ms += cpu_ms;
      usage.processes += 1;
      if pid.as_u32() == root {
        usage.app_memory = process.memory();
        usage.app_cpu_ms = cpu_ms;
      }
    }

    if usage.processes == 0 {
      anyhow::bail!("process {root} is not running");
    }

    Ok(usage)
  }

  pub fn kill_tree(root: u32) {
    let Ok(system) = system() else {
      return;
    };
    for pid in tree(&system, root) {
      if let Some(process) = system.process(pid) {
        process.kill();
      }
    }
  }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
  use super::Usage;
  use anyhow::Result;

  // On macOS the WebKit processes are XPC services started by launchd: they are not
  // descendants of the app, so they can't be attributed to it from the outside.
  pub fn usage(_root: u32) -> Result<Usage> {
    anyhow::bail!("resource usage is not implemented for this platform")
  }

  pub fn kill_tree(_root: u32) {}
}
