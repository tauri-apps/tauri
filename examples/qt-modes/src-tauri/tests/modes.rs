// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runs `qt-modes` once per Tauri mode (see `src/main.rs`), offscreen unless `QT_QPA_PLATFORM` is
//! set, and checks that every IPC check passed. The modes run one at a time: they share fixed
//! ports and the app's profile.

use std::{
  io::Read,
  process::{Command, Stdio},
  sync::Mutex,
  thread,
  time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(180);
static SERIAL: Mutex<()> = Mutex::new(());

fn run(mode: &str) {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let mut cmd = Command::new(env!("CARGO_BIN_EXE_qt-modes"));
  cmd
    .env("QT_MODES_MODE", mode)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  if std::env::var_os("QT_QPA_PLATFORM").is_none() {
    cmd.env("QT_QPA_PLATFORM", "offscreen");
  }
  let mut child = cmd.spawn().expect("spawn qt-modes");
  let drain = |mut pipe: Box<dyn Read + Send>| {
    thread::spawn(move || {
      let mut s = String::new();
      let _ = pipe.read_to_string(&mut s);
      s
    })
  };
  let stdout = drain(Box::new(child.stdout.take().unwrap()));
  let stderr = drain(Box::new(child.stderr.take().unwrap()));
  let deadline = Instant::now() + TIMEOUT;
  let status = loop {
    if let Some(status) = child.try_wait().unwrap() {
      break Some(status);
    }
    if Instant::now() > deadline {
      let _ = child.kill();
      let _ = child.wait();
      break None;
    }
    thread::sleep(Duration::from_millis(50));
  };
  let stdout = stdout.join().unwrap();
  let stderr = stderr.join().unwrap();
  let passes = stdout.lines().filter(|l| l.starts_with("[PASS]")).count();
  let failures: Vec<_> = stdout.lines().filter(|l| l.starts_with("[FAIL]")).collect();
  let dump = || {
    let tail: Vec<_> = stderr
      .lines()
      .rev()
      .take(40)
      .collect::<Vec<_>>()
      .into_iter()
      .rev()
      .collect();
    format!(
      "--- stdout ---\n{stdout}\n--- stderr (tail) ---\n{}",
      tail.join("\n")
    )
  };
  assert!(status.is_some(), "{mode}: timed out\n{}", dump());
  assert!(
    failures.is_empty(),
    "{mode}: {} check(s) failed\n{}",
    failures.len(),
    dump()
  );
  assert!(
    stdout.contains("[DONE] 0 failure(s)"),
    "{mode}: the checks didn't finish\n{}",
    dump()
  );
  assert!(
    passes >= 12,
    "{mode}: only {passes} checks reported\n{}",
    dump()
  );
  assert_eq!(status.unwrap().code(), Some(0), "{mode}\n{}", dump());
}

#[test]
fn brownfield() {
  run("brownfield");
}

#[test]
fn isolation() {
  run("isolation");
}

#[test]
fn devserver() {
  run("devserver");
}

#[test]
fn postmessage() {
  run("postmessage");
}

#[test]
fn noglobal() {
  run("noglobal");
}

#[test]
fn remote() {
  run("remote");
}
