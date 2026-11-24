// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A test harness for code that drives a Qt application (feature `test-harness`).
//!
//! Qt allows one `QApplication` per process, on the main thread, and `qtshim` deliberately can't
//! create it twice. libtest runs tests on worker threads of one process, so Qt tests use this
//! harness instead (`harness = false`): every [`Case`] runs in a fresh child process (the test
//! binary re-executing itself), on that process's main thread. That isolates crashes, panics and
//! shutdown, and lets a case assert that something panics or that the process survives.
//!
//! Each child runs with `QT_QPA_PLATFORM=offscreen` unless the variable is already set, so tests
//! need no display.
//!
//! ```ignore
//! fn main() {
//!   qtshim::test_harness::main(
//!     vec![Case::pass("app::init returns a description", app_init)],
//!     || if qtshim::app::is_running() { qtshim::app::shutdown() },
//!   );
//! }
//! ```
//!
//! At most 2 cases run at once by default (each is a Qt/Chromium process tree); override with
//! `--test-threads N` or `QT_TEST_THREADS=N`. Children get
//! `QTWEBENGINE_CHROMIUM_FLAGS=--renderer-process-limit=1` unless the variable is set.
//!
//! The usual libtest arguments work (filters, `--exact`, `--ignored`, `--list`, `--nocapture`,
//! `--test-threads`). With `--nocapture` each child's output is printed.
//!
//! The harness itself doesn't touch Qt: per-process setup and teardown come from the caller.

use std::{
  io::Read,
  panic::{AssertUnwindSafe, catch_unwind},
  process::{Command, Stdio},
  thread,
  time::{Duration, Instant},
};

use libtest_mimic::{Arguments, Failed, Trial};

/// The environment variable that tells a child process which case to run.
const CASE_ENV: &str = "QT_TEST_HARNESS_CASE";

/// How long a case may run by default.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// What a case's process must do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
  /// Return normally.
  Pass,
  /// Panic, with a message containing this text.
  Panic(&'static str),
}

/// One test, run in its own process.
#[derive(Debug, Clone, Copy)]
pub struct Case {
  /// The test name shown by libtest (and matched by filters).
  pub name: &'static str,
  /// The test body. It runs on the child's main thread; the teardown passed to [`main`] runs
  /// after it (also after a panic).
  pub run: fn(),
  /// The expected outcome.
  pub expect: Expect,
  /// Kill the process after this long.
  pub timeout: Duration,
  /// Skip unless `--ignored` / `--include-ignored` is passed.
  pub ignored: bool,
}

impl Case {
  /// A case that must return normally.
  pub const fn pass(name: &'static str, run: fn()) -> Self {
    Self {
      name,
      run,
      expect: Expect::Pass,
      timeout: DEFAULT_TIMEOUT,
      ignored: false,
    }
  }

  /// A case that must panic with a message containing `message`.
  pub const fn panics(name: &'static str, message: &'static str, run: fn()) -> Self {
    Self {
      name,
      run,
      expect: Expect::Panic(message),
      timeout: DEFAULT_TIMEOUT,
      ignored: false,
    }
  }

  /// The same case with another timeout.
  #[must_use]
  pub const fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// The same case, ignored by default (e.g. a known bug, or needs real hardware).
  #[must_use]
  pub const fn ignored(mut self) -> Self {
    self.ignored = true;
    self
  }
}

/// Run `cases`, then exit. Call it from the `main` of a `harness = false` test.
///
/// `teardown` runs in each child after its case (even if the case panicked), e.g. to shut the
/// application down cleanly so the process doesn't exit with Qt still alive.
pub fn main(cases: Vec<Case>, teardown: fn()) -> ! {
  check_unique_names(&cases);
  if let Ok(name) = std::env::var(CASE_ENV) {
    run_child(&cases, &name, teardown);
  }
  let mut args = Arguments::from_args();
  // Every case is a whole Qt (often Chromium) process tree, so don't run one per CPU core.
  if args.test_threads.is_none() {
    args.test_threads = Some(default_parallelism());
  }
  let nocapture = args.nocapture;
  let trials = cases
    .into_iter()
    .map(|case| {
      Trial::test(case.name, move || run_parent(case, nocapture)).with_ignored_flag(case.ignored)
    })
    .collect();
  libtest_mimic::run(&args, trials).exit()
}

/// How many cases run at once without `--test-threads`: `QT_TEST_THREADS`, else 2.
fn default_parallelism() -> usize {
  std::env::var("QT_TEST_THREADS")
    .ok()
    .and_then(|v| v.parse().ok())
    .filter(|&n| n > 0)
    .unwrap_or(2)
}

fn check_unique_names(cases: &[Case]) {
  let mut names: Vec<_> = cases.iter().map(|c| c.name).collect();
  names.sort_unstable();
  if let Some(w) = names.windows(2).find(|w| w[0] == w[1]) {
    panic!("qtshim test harness: duplicate case name {:?}", w[0]);
  }
}

/// In the child: run the case on the main thread, then the teardown, then exit.
fn run_child(cases: &[Case], name: &str, teardown: fn()) -> ! {
  let Some(case) = cases.iter().find(|c| c.name == name) else {
    eprintln!("qtshim test harness: no case named {name:?}");
    std::process::exit(2);
  };
  let result = catch_unwind(case.run);
  // The panic hook already printed the message for the parent to match.
  if catch_unwind(AssertUnwindSafe(teardown)).is_err() {
    eprintln!("qtshim test harness: teardown panicked");
    std::process::exit(3);
  }
  // `exit` (not `abort`) so coverage data gets written.
  std::process::exit(if result.is_ok() { 0 } else { 101 });
}

/// In the parent: run the case in a child process and check its outcome.
fn run_parent(case: Case, nocapture: bool) -> Result<(), Failed> {
  let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
  let mut cmd = Command::new(exe);
  cmd
    .env(CASE_ENV, case.name)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  if std::env::var_os("QT_QPA_PLATFORM").is_none() {
    cmd.env("QT_QPA_PLATFORM", "offscreen");
  }
  // One renderer process per case is plenty for tests, and keeps the process count bounded.
  if std::env::var_os("QTWEBENGINE_CHROMIUM_FLAGS").is_none() {
    cmd.env("QTWEBENGINE_CHROMIUM_FLAGS", "--renderer-process-limit=1");
  }
  let mut child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
  let stdout = drain(child.stdout.take().unwrap());
  let stderr = drain(child.stderr.take().unwrap());

  let deadline = Instant::now() + case.timeout;
  let status = loop {
    if let Some(status) = child.try_wait().map_err(|e| format!("wait: {e}"))? {
      break Some(status);
    }
    if Instant::now() >= deadline {
      let _ = child.kill();
      let _ = child.wait();
      break None;
    }
    thread::sleep(Duration::from_millis(10));
  };
  let stdout = stdout.join().unwrap_or_default();
  let stderr = stderr.join().unwrap_or_default();
  if nocapture {
    print!("{stdout}");
    eprint!("{stderr}");
  }

  let outcome = match status {
    None => Err(format!("timed out after {:?}", case.timeout)),
    Some(status) => match (case.expect, status.code()) {
      (Expect::Pass, Some(0)) => Ok(()),
      (Expect::Pass, _) => Err(format!("failed ({status})")),
      (Expect::Panic(msg), Some(101)) if stderr.contains(msg) => Ok(()),
      (Expect::Panic(msg), Some(101)) => Err(format!("panicked, but not with {msg:?}")),
      (Expect::Panic(msg), _) => Err(format!("expected a panic containing {msg:?}, got {status}")),
    },
  };
  outcome.map_err(|why| Failed::from(format!("{why}\n{}", tail(&stdout, &stderr))))
}

fn drain(mut pipe: impl Read + Send + 'static) -> thread::JoinHandle<String> {
  thread::spawn(move || {
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
  })
}

/// The end of the child's output, for failure messages.
fn tail(stdout: &str, stderr: &str) -> String {
  fn last_lines(s: &str, n: usize) -> String {
    let lines: Vec<_> = s.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
  }
  format!(
    "--- stdout (tail)\n{}\n--- stderr (tail)\n{}",
    last_lines(stdout, 20),
    last_lines(stderr, 40)
  )
}
