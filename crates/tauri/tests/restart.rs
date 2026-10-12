// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `tauri::process::restart` exits the process, so this test copies its own test binary into an
//! app bundle and runs it there. The same test function then plays three roles, told apart by a
//! state file next to the bundle:
//!
//! - no state file: the test itself, which launches the bundled copy
//! - state `launched`: the first instance, which calls `restart`
//! - state `restarted`: the relaunched instance, which reports what it inherited

#![cfg(target_os = "macos")]

use std::{
  fs,
  io::Write,
  path::Path,
  process::{Command, Stdio},
  time::{Duration, Instant},
};

const TEST_NAME: &str = "restart_relaunches_bundle_without_inheriting_stdio";

fn wait_for(path: &Path, timeout: Duration) -> bool {
  let start = Instant::now();
  while start.elapsed() < timeout {
    if path.exists() {
      return true;
    }
    std::thread::sleep(Duration::from_millis(50));
  }
  false
}

fn process_group(pid: u32) -> String {
  let output = Command::new("ps")
    .args(["-o", "pgid=", "-p", &pid.to_string()])
    .output()
    .unwrap();
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn restart_relaunches_bundle_without_inheriting_stdio() {
  let exe = std::env::current_exe().unwrap();
  // <dir>/RestartTest.app/Contents/MacOS/<exe>
  if exe
    .parent()
    .unwrap()
    .ends_with("RestartTest.app/Contents/MacOS")
  {
    return run_bundled(exe.ancestors().nth(4).unwrap());
  }

  // canonicalized because `restart` refuses to relaunch a binary whose path has symlinks,
  // and the default temp dir lives under the `/var` -> `/private/var` symlink
  let dir = std::env::temp_dir()
    .canonicalize()
    .unwrap()
    .join(format!("tauri-restart-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&dir);
  let macos_dir = dir.join("RestartTest.app/Contents/MacOS");
  fs::create_dir_all(&macos_dir).unwrap();
  let bundled_exe = macos_dir.join(exe.file_name().unwrap());
  fs::copy(&exe, &bundled_exe).unwrap();
  fs::write(
    dir.join("RestartTest.app/Contents/Info.plist"),
    format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>{}</string>
<key>CFBundleIdentifier</key><string>app.tauri.restart-test-{}</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>"#,
      exe.file_name().unwrap().to_string_lossy(),
      std::process::id()
    ),
  )
  .unwrap();
  fs::write(dir.join("state"), "launched").unwrap();

  // like an IDE or launcher script: pipe the app's output, wait for it to exit, stop reading
  let mut first = Command::new(&bundled_exe)
    .args([TEST_NAME, "--exact", "--test-threads=1"])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  fs::write(dir.join("first-pgid"), process_group(first.id())).unwrap();
  let status = first.wait().unwrap();
  assert!(status.success(), "first instance failed: {status}");
  drop(first.stdout.take());
  drop(first.stderr.take());
  fs::write(dir.join("pipes-closed"), "").unwrap();

  let report_path = dir.join("report");
  let relaunched = wait_for(&report_path, Duration::from_secs(30));
  let report = fs::read_to_string(&report_path).unwrap_or_default();
  let _ = fs::remove_dir_all(&dir);

  assert!(
    relaunched,
    "the relaunched instance never reported back: it was not started, \
     or it died writing to the stdio it inherited from the first instance"
  );
  assert_eq!(report, "ok", "relaunched instance: {report}");
}

fn run_bundled(dir: &Path) {
  match fs::read_to_string(dir.join("state")).unwrap().as_str() {
    "launched" => {
      fs::write(dir.join("state"), "restarted").unwrap();
      tauri::process::restart(&tauri::Env::default());
    }
    "restarted" => {
      let mut problems = Vec::new();

      let args: Vec<_> = std::env::args().skip(1).collect();
      if args.first().map(String::as_str) != Some(TEST_NAME) {
        problems.push(format!("arguments were not forwarded: {args:?}"));
      }

      if wait_for(&dir.join("pipes-closed"), Duration::from_secs(10)) {
        // bypass the test harness output capture, a real app writes straight to the fd
        if let Err(e) = std::io::stdout().write_all(b"hello from the relaunched app\n") {
          problems.push(format!("writing to stdout failed: {e}"));
        }
      } else {
        problems.push("the test never closed the first instance's pipes".into());
      }

      let first_pgid = fs::read_to_string(dir.join("first-pgid")).unwrap_or_default();
      if process_group(std::process::id()) == first_pgid {
        problems.push(format!(
          "still in the first instance's process group {first_pgid}"
        ));
      }

      let report = if problems.is_empty() {
        "ok".to_string()
      } else {
        problems.join("; ")
      };
      fs::write(dir.join("report"), report).unwrap();
    }
    state => panic!("unexpected state {state}"),
  }
}
