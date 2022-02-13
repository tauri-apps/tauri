// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Helper for generic catch-all errors.
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

/// https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--1300-1699-
#[cfg(windows)]
const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

/// Represents a successfully created symlink.
enum Symlink {
  /// Path to the created symlink
  Created(PathBuf),

  /// A symlink that failed due to missing permissions (Windows).
  #[allow(dead_code)]
  Privilege,
}

/// Compile the nested restart binary so that we have access to it during the test.
///
/// This is compiled inside of this test so that we always have a literal binary file we can use
/// for some filesystem related tests. Because of how integration tests work, the current working
/// directory should be the same as the manifest of the crate (not workspace) this integration test
/// is a part of.
fn compile_restart_test_binary() -> io::Result<PathBuf> {
  let project = PathBuf::from("tests").join("restart");

  let mut cargo = Command::new("cargo");
  cargo.arg("build");
  cargo.arg("--manifest-path");
  cargo.arg(project.join("Cargo.toml"));

  // enable the dangerous macos flag on tauri if the test runner has the feature enabled
  if cfg!(feature = "process-relaunch-dangerous-allow-symlink-macos") {
    cargo.args([
      "--features",
      "tauri/process-relaunch-dangerous-allow-symlink-macos",
    ]);
  }

  let status = cargo.status()?;
  if !status.success() {
    return Err(io::Error::new(
      io::ErrorKind::Other,
      "Unable to compile restart test cargo project inside restart integration test",
    ));
  }

  let profile = if cfg!(debug_assertions) {
    "debug"
  } else {
    "release"
  };

  let bin = if cfg!(windows) {
    "restart.exe"
  } else {
    "restart"
  };

  Ok(project.join("target").join(profile).join(bin))
}

/// Compile the test binary, run it, and compare it with expected output.
///
/// Failing to create a symlink due to permissions issues is also a success
/// for the purpose of this runner.
fn symlink_runner(create_symlinks: impl Fn(&Path) -> io::Result<Symlink>) -> Result {
  let compiled_binary = compile_restart_test_binary()?;

  // set up all the temporary file paths
  let temp = tempfile::TempDir::new()?;
  let bin = temp.path().canonicalize()?.join("restart.exe");

  // copy the built restart test binary to our temporary directory
  std::fs::copy(compiled_binary, &bin)?;

  if let Symlink::Created(link) = create_symlinks(&bin)? {
    // run the command from the symlink, so that we can test if restart resolves it correctly
    let mut cmd = Command::new(link);

    // add the restart parameter so that the invocation will call tauri::api::process::restart
    cmd.arg("restart");

    let output = cmd.output()?;

    // run `TempDir` destructors to prevent resource leaking if the assertion fails
    drop(temp);

    if output.status.success() {
      // gather the output into a string
      let stdout = String::from_utf8(output.stdout)?;

      // we expect the output to be the bin path, twice
      assert_eq!(stdout, format!("{bin}\n{bin}\n", bin = bin.display()));
    } else if cfg!(all(
      target_os = "macos",
      not(feature = "process-relaunch-dangerous-allow-symlink-macos")
    )) {
      // we expect this to fail on macOS without the dangerous symlink flag set
      let stderr = String::from_utf8(output.stderr)?;

      // make sure it's the error that we expect
      assert!(stderr.contains(
        "StartingBinary found current_exe() that contains a symlink on a non-allowed platform"
      ));
    } else {
      // we didn't expect the program to fail in this configuration, just panic
      panic!("restart integration test runner failed for unknown reason");
    }
  }

  Ok(())
}

/// Cross-platform way to create a symlink
///
/// Symlinks that failed to create due to permissions issues (like on Windows)
/// are also seen as successful for the purpose of this testing suite.
fn create_symlink(original: &Path, link: PathBuf) -> io::Result<Symlink> {
  #[cfg(unix)]
  return std::os::unix::fs::symlink(original, &link).map(|()| Symlink::Created(link));

  #[cfg(windows)]
  return match std::os::windows::fs::symlink_file(original, &link) {
    Ok(()) => Ok(Symlink::Created(link)),
    Err(e) => match e.raw_os_error() {
      Some(ERROR_PRIVILEGE_NOT_HELD) => Ok(Symlink::Privilege),
      _ => Err(e),
    },
  };
}

/// Only use 1 test to prevent cargo from waiting on itself.
///
/// While not ideal, this is fine because they use the same solution for both cases.
#[test]
fn restart_symlinks() -> Result {
  // single symlink
  symlink_runner(|bin| {
    let mut link = bin.to_owned();
    link.set_file_name("symlink");
    link.set_extension("exe");
    create_symlink(bin, link)
  })?;

  // nested symlinks
  symlink_runner(|bin| {
    let mut link1 = bin.to_owned();
    link1.set_file_name("symlink1");
    link1.set_extension("exe");
    create_symlink(bin, link1.clone())?;

    let mut link2 = bin.to_owned();
    link2.set_file_name("symlink2");
    link2.set_extension("exe");
    create_symlink(&link1, link2)
  })
}
