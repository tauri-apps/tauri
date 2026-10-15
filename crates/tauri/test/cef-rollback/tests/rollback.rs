// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Rolls a CEF application back to an older Chromium milestone on a profile it already
//! has, and forward again, on the real runtime.
//!
//! One CEF distribution is linked into the application, so the test cannot run two
//! Chromium builds; it stages what a rollback looks like on disk instead. The runtime
//! keeps a `Last Version` breadcrumb in the root cache path naming the Chromium that ran
//! last, and a release rolled back to an older CEF finds a higher milestone in it.
//! Rewriting that file to a higher milestone between two launches of the one binary is
//! that situation, as far as the runtime can tell.
//!
//! What a launch found in the profile comes from the page (`dist/index.html`): it keeps
//! a launch counter in `localStorage`, which Chromium stores in the profile, and reports
//! it back before the application exits. A kept profile hands the next launch its
//! counter; a reset one hands it nothing.
//!
//! The application needs the CEF distribution around its executable. On macOS that is
//! the `.app` layout CEF launches its helper processes from, which the test assembles
//! from the distribution `cef-dll-sys` resolved at build time; on Linux and Windows
//! `cef-dll-sys` copies the distribution next to the executable, which runs as it is.

use std::{
  fs,
  path::{Path, PathBuf},
  process::{Child, Command, ExitStatus, Stdio},
  sync::{
    Mutex, MutexGuard, OnceLock,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use serde::Deserialize;

/// The breadcrumb `tauri-runtime-cef` keeps in the root cache path.
const LAST_VERSION_FILE: &str = "Last Version";
/// The suffix of a root cache path a reset moved aside for deletion.
const MOVED_ASIDE_SUFFIX: &str = ".tauri-delete";

// The application's environment, as `src/main.rs` reads it.
const ROOT_CACHE_PATH: &str = "CEF_ROLLBACK_ROOT_CACHE_PATH";
const DOWNGRADE: &str = "CEF_ROLLBACK_DOWNGRADE";
const REPORT: &str = "CEF_ROLLBACK_REPORT";
const HOLD: &str = "CEF_ROLLBACK_HOLD";

/// A CEF start on a cold disk cache takes a few seconds, more on a loaded CI runner.
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(120);
const EXIT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(100);

/// What the application wrote to its report file: what the page found in the profile.
#[derive(Debug, Deserialize)]
struct Report {
  /// The Chromium the binary embeds, `MAJOR.MINOR.BUILD.PATCH`.
  chromium_version: String,
  /// The page's launch counter before this launch bumped it.
  previous_runs: u32,
  /// Whether the cookie every launch sets was there.
  cookie_seen: bool,
}

/// One test at a time. Every test launches the application, and two instances on one
/// desktop share the GPU process, the window server and the tester's patience.
fn serial() -> MutexGuard<'static, ()> {
  static LOCK: Mutex<()> = Mutex::new(());
  LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The application executable, laid out as CEF needs it, prepared once per test process.
fn executable() -> &'static Path {
  static EXECUTABLE: OnceLock<PathBuf> = OnceLock::new();
  EXECUTABLE.get_or_init(|| layout::executable(Path::new(env!("CARGO_BIN_EXE_cef-rollback"))))
}

/// A root cache path of its own for one test, with a parent directory of its own too, so
/// a profile moved aside by a reset (a sibling of the root) can be looked for.
struct Profile {
  dir: tempfile::TempDir,
  root: PathBuf,
}

impl Profile {
  fn new() -> Self {
    let dir = tempfile::Builder::new()
      .prefix("cef-rollback-")
      .tempdir()
      .expect("failed to create the profile's temporary directory");
    let root = dir.path().join("cef");
    fs::create_dir_all(&root).unwrap();
    Self { dir, root }
  }

  fn last_version(&self) -> String {
    fs::read_to_string(self.root.join(LAST_VERSION_FILE))
      .expect("the runtime left no Last Version file in the root cache path")
      .trim()
      .to_string()
  }

  /// Stages what a launch by another release leaves behind: its own version in the
  /// breadcrumb.
  fn stage_last_version(&self, version: &str) {
    fs::write(self.root.join(LAST_VERSION_FILE), version).unwrap();
  }

  /// A file of ours inside the profile, to tell a kept directory from a recreated one
  /// independently of anything Chromium does.
  fn sentinel(&self) -> PathBuf {
    self.root.join("sentinel-planted-by-the-test")
  }

  fn plant_sentinel(&self) {
    fs::write(self.sentinel(), b"").unwrap();
  }

  /// Profiles a reset moved aside and has not deleted yet.
  fn moved_aside(&self) -> Vec<PathBuf> {
    fs::read_dir(self.dir.path())
      .unwrap()
      .flatten()
      .map(|entry| entry.path())
      .filter(|path| {
        path.is_dir()
          && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cef.") && name.ends_with(MOVED_ASIDE_SUFFIX))
      })
      .collect()
  }

  fn diagnostics(&self) -> String {
    let cef_log = self.root.join("cef.log");
    format!(
      "root cache path: {}\n--- cef.log ---\n{}",
      self.root.display(),
      tail(&cef_log)
    )
  }
}

/// One launch of the application.
struct Instance {
  child: Child,
  report: PathBuf,
  stdout: PathBuf,
  stderr: PathBuf,
}

impl Instance {
  fn spawn(profile: &Profile, policy: Option<&str>, hold: bool) -> Self {
    static LAUNCHES: AtomicUsize = AtomicUsize::new(0);
    let launch = LAUNCHES.fetch_add(1, Ordering::Relaxed);
    let report = profile.dir.path().join(format!("report-{launch}.json"));
    let stdout = profile.dir.path().join(format!("stdout-{launch}.log"));
    let stderr = profile.dir.path().join(format!("stderr-{launch}.log"));

    let mut command = Command::new(executable());
    command
      .env(ROOT_CACHE_PATH, &profile.root)
      .env(REPORT, &report)
      .env_remove(DOWNGRADE)
      .env_remove(HOLD)
      .stdin(Stdio::null())
      .stdout(fs::File::create(&stdout).unwrap())
      .stderr(fs::File::create(&stderr).unwrap());
    if let Some(policy) = policy {
      command.env(DOWNGRADE, policy);
    }
    if hold {
      command.env(HOLD, "1");
    }
    let child = command
      .spawn()
      .unwrap_or_else(|error| panic!("failed to launch {}: {error}", executable().display()));
    Self {
      child,
      report,
      stdout,
      stderr,
    }
  }

  /// Blocks until the page has reported, the process has exited without reporting, or
  /// the launch timed out; the two latter fail the test with everything the run logged.
  fn wait_for_report(&mut self, profile: &Profile) -> Report {
    let started = Instant::now();
    loop {
      if let Ok(contents) = fs::read(&self.report) {
        return serde_json::from_slice(&contents)
          .expect("the application wrote a malformed report");
      }
      match self.child.try_wait().unwrap() {
        Some(status) => self.fail(profile, &format!("exited with {status} before reporting")),
        None if started.elapsed() > LAUNCH_TIMEOUT => {
          let _ = self.child.kill();
          self.fail(profile, "did not report in time")
        }
        None => std::thread::sleep(POLL),
      }
    }
  }

  fn wait_for_exit(&mut self, profile: &Profile) -> ExitStatus {
    let started = Instant::now();
    loop {
      if let Some(status) = self.child.try_wait().unwrap() {
        return status;
      }
      if started.elapsed() > EXIT_TIMEOUT {
        let _ = self.child.kill();
        self.fail(profile, "did not exit in time");
      }
      std::thread::sleep(POLL);
    }
  }

  /// Ends the instance the way a crash would: no clean shutdown, no cleanup.
  fn kill(mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }

  fn fail(&mut self, profile: &Profile, what: &str) -> ! {
    let _ = self.child.wait();
    panic!(
      "the application {what}\n--- stdout ---\n{}\n--- stderr ---\n{}\n{}",
      tail(&self.stdout),
      tail(&self.stderr),
      profile.diagnostics()
    );
  }
}

impl Drop for Instance {
  fn drop(&mut self) {
    // Whatever a test did with it, no instance outlives the test.
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// Launches the application on the profile, lets the page report, and waits for the exit.
fn launch(profile: &Profile, policy: Option<&str>) -> Report {
  let mut instance = Instance::spawn(profile, policy, false);
  let report = instance.wait_for_report(profile);
  let status = instance.wait_for_exit(profile);
  if !status.success() {
    instance.fail(profile, &format!("exited with {status} after reporting"));
  }
  report
}

fn tail(path: &Path) -> String {
  const LINES: usize = 40;
  let contents = fs::read_to_string(path).unwrap_or_default();
  let lines: Vec<&str> = contents.lines().collect();
  lines[lines.len().saturating_sub(LINES)..].join("\n")
}

fn wait_until(what: &str, mut condition: impl FnMut() -> bool) {
  let started = Instant::now();
  while !condition() {
    assert!(
      started.elapsed() < EXIT_TIMEOUT,
      "timed out waiting until {what}"
    );
    std::thread::sleep(POLL);
  }
}

fn components(version: &str) -> [u32; 4] {
  let parsed: Vec<u32> = version
    .split('.')
    .map(|component| component.parse().expect("a numeric version component"))
    .collect();
  parsed
    .try_into()
    .unwrap_or_else(|_| panic!("expected MAJOR.MINOR.BUILD.PATCH, got {version:?}"))
}

/// The version a release on the next Chromium milestone would leave in the breadcrumb.
fn next_milestone(version: &str) -> String {
  let [major, ..] = components(version);
  format!("{}.0.0.0", major + 1)
}

/// A later build of the milestone this binary embeds, as a patch release of the same
/// CEF branch would leave.
fn later_build_of_the_same_milestone(version: &str) -> String {
  let [major, minor, build, patch] = components(version);
  format!("{major}.{minor}.{}.{patch}", build + 1)
}

#[test]
fn a_launch_records_the_chromium_it_ran_on() {
  let _serial = serial();
  let profile = Profile::new();

  let report = launch(&profile, None);

  assert_eq!(report.previous_runs, 0, "a fresh profile has no counter");
  assert!(!report.cookie_seen);
  let [major, ..] = components(&report.chromium_version);
  assert!(
    major >= 100,
    "not a Chromium version: {}",
    report.chromium_version
  );
  assert_eq!(
    profile.last_version(),
    report.chromium_version,
    "the breadcrumb names the Chromium that ran"
  );
  assert!(
    profile.root.join("Default").is_dir(),
    "Chromium created its profile inside the root cache path"
  );
}

#[test]
fn a_relaunch_on_the_same_chromium_keeps_the_profile() {
  let _serial = serial();
  let profile = Profile::new();

  let first = launch(&profile, None);
  let second = launch(&profile, None);

  assert_eq!(second.previous_runs, 1, "the counter survived the relaunch");
  assert!(second.cookie_seen, "the cookie survived the relaunch");
  assert_eq!(profile.last_version(), first.chromium_version);
  assert!(profile.moved_aside().is_empty());
}

#[test]
fn a_rollback_keeps_the_profile_unless_asked_otherwise() {
  let _serial = serial();
  let profile = Profile::new();
  let first = launch(&profile, None);
  let newer = next_milestone(&first.chromium_version);

  // A release on the next milestone ran here; this binary is the rollback. The runtime's
  // default applies: what Chrome does, which is to run on the profile as it is.
  profile.stage_last_version(&newer);
  profile.plant_sentinel();
  let second = launch(&profile, None);

  assert_eq!(
    second.previous_runs, 1,
    "the profile was kept, counter and all"
  );
  assert!(second.cookie_seen);
  assert!(
    profile.sentinel().exists(),
    "nothing in the profile was touched"
  );
  assert_eq!(
    profile.last_version(),
    first.chromium_version,
    "the breadcrumb now names the Chromium that ran last"
  );
  assert!(profile.moved_aside().is_empty());

  // The same, asked for explicitly.
  profile.stage_last_version(&newer);
  let third = launch(&profile, Some("keep"));
  assert_eq!(third.previous_runs, 2);
  assert!(profile.sentinel().exists());
  assert_eq!(profile.last_version(), first.chromium_version);
}

#[test]
fn reset_profile_starts_clean_on_a_rollback() {
  let _serial = serial();
  let profile = Profile::new();
  let first = launch(&profile, None);

  profile.stage_last_version(&next_milestone(&first.chromium_version));
  profile.plant_sentinel();
  let second = launch(&profile, Some("reset"));

  assert_eq!(
    second.previous_runs, 0,
    "the counter went away with the profile the newer Chromium wrote"
  );
  assert!(!second.cookie_seen);
  assert!(
    !profile.sentinel().exists(),
    "the root cache path was recreated, not cleaned in place"
  );
  assert_eq!(
    profile.last_version(),
    first.chromium_version,
    "the fresh profile is stamped with the Chromium that created it"
  );

  // The old profile is deleted in the background: by the launch that moved it aside if
  // that got to it before exiting, otherwise by the next one.
  let third = launch(&profile, Some("reset"));
  assert_eq!(
    third.previous_runs, 1,
    "the fresh profile is an ordinary profile from then on"
  );
  wait_until("the profile moved aside is deleted", || {
    profile.moved_aside().is_empty()
  });
}

#[test]
fn a_rollback_within_the_milestone_keeps_the_profile_even_when_resets_are_asked_for() {
  let _serial = serial();
  let profile = Profile::new();
  let first = launch(&profile, Some("reset"));

  // A patch release of the same CEF branch ran here. Chrome supports this downgrade, so
  // the runtime leaves the profile alone whatever the policy says.
  profile.stage_last_version(&later_build_of_the_same_milestone(&first.chromium_version));
  profile.plant_sentinel();
  let second = launch(&profile, Some("reset"));

  assert_eq!(second.previous_runs, 1);
  assert!(profile.sentinel().exists());
  assert_eq!(profile.last_version(), first.chromium_version);
  assert!(profile.moved_aside().is_empty());
}

#[test]
fn a_running_instance_keeps_its_profile_through_a_rollback() {
  let _serial = serial();
  let profile = Profile::new();

  // An instance that stays up, holding the profile.
  let mut running = Instance::spawn(&profile, None, true);
  let first = running.wait_for_report(&profile);
  let newer = next_milestone(&first.chromium_version);

  // A rollback launched on a profile another instance holds. Chromium's process
  // singleton hands the launch over to the running instance, and the runtime must not
  // have touched the profile on the way there. The runtime sees the singleton lock on
  // Unix; on Windows the running instance's open files make the move itself fail.
  profile.stage_last_version(&newer);
  profile.plant_sentinel();
  let mut second = Instance::spawn(&profile, Some("reset"), false);
  second.wait_for_exit(&profile);

  assert!(
    profile.sentinel().exists(),
    "the profile of a running instance is never moved aside"
  );
  assert_eq!(
    profile.last_version(),
    newer,
    "the reset stays pending: the breadcrumb still names the newer Chromium"
  );
  assert!(profile.moved_aside().is_empty());

  // The running instance goes away the way a crash takes it, singleton lock left behind.
  running.kill();
  let third = launch(&profile, Some("reset"));

  assert_eq!(
    third.previous_runs, 0,
    "with the holder gone the pending reset goes through, stale lock notwithstanding"
  );
  assert!(!profile.sentinel().exists());
  assert_eq!(profile.last_version(), first.chromium_version);
}

/// Lays the application out as CEF needs it on this platform.
///
/// CEF on macOS runs from an `.app`: the runtime loads the framework from
/// `Contents/Frameworks` next to the executable, and Chromium launches its renderer,
/// GPU and utility processes from helper apps in the same directory, named after the
/// executable. The bundler builds a dedicated helper executable for a shipped app; here
/// the application binary serves as its own helper, which is what its `cef_entry_point`
/// is for (and what it is on Linux and Windows).
#[cfg(target_os = "macos")]
mod layout {
  use std::{fs, path::Path, path::PathBuf, process::Command};

  const FRAMEWORK: &str = "Chromium Embedded Framework.framework";
  const HELPER_SUFFIXES: [&str; 5] = ["", " (GPU)", " (Renderer)", " (Plugin)", " (Alerts)"];

  pub fn executable(built: &Path) -> PathBuf {
    let name = built.file_name().unwrap().to_str().unwrap();
    let bundle = Path::new(env!("CARGO_TARGET_TMPDIR"))
      .join("cef-rollback")
      .join(format!("{name}.app"));
    let contents = bundle.join("Contents");
    let frameworks = contents.join("Frameworks");

    // Rebuilt from scratch every test process: the binary and the distribution may both
    // have changed since the last one.
    let _ = fs::remove_dir_all(&bundle);
    fs::create_dir_all(contents.join("MacOS")).unwrap();
    fs::create_dir_all(&frameworks).unwrap();

    let executable = contents.join("MacOS").join(name);
    place(built, &executable);
    fs::write(
      contents.join("Info.plist"),
      info_plist(name, "app.tauri.test.cef-rollback", false),
    )
    .unwrap();

    // `cp` keeps the framework exactly as shipped, and clones it where the file system can.
    let status = Command::new("cp")
      .arg("-Rc")
      .arg(cef_dir().join(FRAMEWORK))
      .arg(&frameworks)
      .status()
      .expect("failed to run cp");
    assert!(
      status.success(),
      "failed to copy {FRAMEWORK} into the bundle"
    );

    for suffix in HELPER_SUFFIXES {
      let helper = format!("{name} Helper{suffix}");
      let helper_contents = frameworks.join(format!("{helper}.app")).join("Contents");
      fs::create_dir_all(helper_contents.join("MacOS")).unwrap();
      place(built, &helper_contents.join("MacOS").join(&helper));
      fs::write(
        helper_contents.join("Info.plist"),
        info_plist(&helper, "app.tauri.test.cef-rollback.helper", true),
      )
      .unwrap();
    }

    executable
  }

  /// The CEF distribution the application was built against, from the crate's build
  /// script (`DEP_TAURI_RUNTIME_CEF_CEF_DIR`), or `CEF_PATH` when pointing at one.
  fn cef_dir() -> PathBuf {
    let candidates = [
      option_env!("CEF_ROLLBACK_CEF_DIR").map(PathBuf::from),
      std::env::var_os("CEF_PATH").map(PathBuf::from),
    ];
    candidates
      .into_iter()
      .flatten()
      .find(|dir| dir.join(FRAMEWORK).is_dir())
      .unwrap_or_else(|| {
        panic!(
          "no CEF binary distribution found: build this crate so `cef-dll-sys` resolves one, or point CEF_PATH at an extracted distribution containing {FRAMEWORK}"
        )
      })
  }

  /// The linker signed the binary ad hoc, and a hard link is the same file, signature
  /// included; a copy carries it along as well.
  fn place(built: &Path, at: &Path) {
    if fs::hard_link(built, at).is_err() {
      fs::copy(built, at).expect("failed to copy the application binary into the bundle");
    }
  }

  fn info_plist(executable: &str, identifier: &str, background_only: bool) -> String {
    let ui_element = if background_only {
      "\n  <key>LSUIElement</key>\n  <true/>"
    } else {
      ""
    };
    format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>English</string>
  <key>CFBundleExecutable</key>
  <string>{executable}</string>
  <key>CFBundleIdentifier</key>
  <string>{identifier}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{executable}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>{ui_element}
</dict>
</plist>
"#
    )
  }
}

/// On Linux and Windows the executable runs where cargo put it: `cef-dll-sys` copies the
/// CEF distribution next to it, and `tauri-build` gives the Linux binary an `$ORIGIN`
/// rpath to find `libcef.so` there.
#[cfg(not(target_os = "macos"))]
mod layout {
  use std::path::{Path, PathBuf};

  pub fn executable(built: &Path) -> PathBuf {
    built.to_path_buf()
  }
}
