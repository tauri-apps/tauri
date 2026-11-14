// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! End-to-end tests: run the built `qt-app` once per `QT_APP_AUTO` mode and check that it exits
//! with code 0 and reports no `[FAIL]` lines.
//!
//! The app runs offscreen (`QT_QPA_PLATFORM=offscreen`) unless `QT_QPA_PLATFORM` is already set,
//! so checks that need a real display report `[SKIP]` instead of failing. Checks for known bugs
//! report `[KNOWN-BUG]` without running (set `QT_APP_BUGS=1` to run them; some hang the app).

use std::{
  io::Read,
  process::{Command, Stdio},
  sync::Mutex,
  thread,
  time::{Duration, Instant},
};

/// Generous: the first page load starts Chromium, and the self-test opens several windows.
const TIMEOUT: Duration = Duration::from_secs(180);

/// Output of one app run.
struct Run {
  code: Option<i32>,
  timed_out: bool,
  stdout: String,
  stderr: String,
}

impl Run {
  fn lines(&self, prefix: &str) -> Vec<&str> {
    self
      .stdout
      .lines()
      .filter(|l| l.starts_with(prefix))
      .collect()
  }

  fn dump(&self) -> String {
    format!(
      "exit code: {:?}{}\n--- stdout ---\n{}\n--- stderr (last 80 lines) ---\n{}",
      self.code,
      if self.timed_out {
        " (killed after timeout)"
      } else {
        ""
      },
      self.stdout,
      tail(&self.stderr, 80),
    )
  }
}

fn tail(s: &str, n: usize) -> String {
  let lines: Vec<_> = s.lines().collect();
  lines[lines.len().saturating_sub(n)..].join("\n")
}

/// The modes share the app's on-disk profile (cookies, cache, Chromium's profile lock), so they
/// run one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn run(mode: &str) -> Run {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let mut cmd = Command::new(env!("CARGO_BIN_EXE_qt-app"));
  cmd
    .env("QT_APP_AUTO", mode)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  if std::env::var_os("QT_QPA_PLATFORM").is_none() {
    cmd.env("QT_QPA_PLATFORM", "offscreen");
  }
  let mut child = cmd.spawn().expect("failed to start qt-app");
  let pipe = |mut r: Box<dyn Read + Send>| {
    thread::spawn(move || {
      let mut buf = Vec::new();
      let _ = r.read_to_end(&mut buf);
      String::from_utf8_lossy(&buf).into_owned()
    })
  };
  let stdout = pipe(Box::new(child.stdout.take().unwrap()));
  let stderr = pipe(Box::new(child.stderr.take().unwrap()));

  let start = Instant::now();
  let (status, timed_out) = loop {
    if let Some(status) = child.try_wait().expect("try_wait") {
      break (Some(status), false);
    }
    if start.elapsed() > TIMEOUT {
      let _ = child.kill();
      break (child.wait().ok(), true);
    }
    thread::sleep(Duration::from_millis(100));
  };
  Run {
    code: status.and_then(|s| s.code()),
    timed_out,
    stdout: stdout.join().unwrap_or_default(),
    stderr: stderr.join().unwrap_or_default(),
  }
}

/// Run `mode`, require a clean exit, no failures and at least `min_checks` passing checks.
fn check(mode: &str, min_checks: usize) -> Run {
  let run = run(mode);
  let fails = run.lines("[FAIL]");
  let passes = run.lines("[PASS]").len();
  let skips = run.lines("[SKIP]");
  let bugs = run.lines("[KNOWN-BUG]");
  assert!(
    !run.timed_out && run.code == Some(0) && fails.is_empty() && passes >= min_checks,
    "QT_APP_AUTO={mode}: {passes} passed, {} failed, {} skipped (want >= {min_checks} passed)\n\
     failures:\n  {}\n{}",
    fails.len(),
    skips.len(),
    fails.join("\n  "),
    run.dump(),
  );
  println!(
    "QT_APP_AUTO={mode}: {passes} passed, {} skipped, {} known bugs",
    skips.len(),
    bugs.len()
  );
  for line in skips.iter().chain(&bugs) {
    println!("  {line}");
  }
  run
}

#[test]
fn selftest() {
  check("1", 60);
}

#[test]
fn smoke() {
  check("smoke", 50);
}

#[test]
fn crash() {
  let run = check("crash", 3);
  assert!(run.stdout.contains("terminated"), "{}", run.dump());
}

#[test]
fn close() {
  let run = check("close", 60);
  for want in [
    "[PASS] exit: ExitRequested(None) after the last window closed, prevented",
    "[PASS] exit: app.exit(0) from another thread after a prevented exit",
    "[run] exit\n",
  ] {
    assert!(
      run.stdout.contains(want),
      "missing {want:?}\n{}",
      run.dump()
    );
  }
}
