// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{io::Cursor, process::Command};

#[derive(Debug, serde::Deserialize)]
struct TargetSpec {
  #[serde(rename = "llvm-target")]
  llvm_target: String,
}

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Try to determine the current target triple.
///
/// Returns a target triple (e.g. `x86_64-unknown-linux-gnu` or `i686-pc-windows-msvc`) or an
/// `Error::Config` if the current config cannot be determined or is not some combination of the
/// following values:
/// `linux, mac, windows` -- `i686, x86, armv7` -- `gnu, musl, msvc`
///
/// * Errors:
///     * Unexpected system config
pub fn target_triple() -> Result<String, crate::Error> {
  let output = Command::new("rustc")
    .args(&["-Z", "unstable-options", "--print", "target-spec-json"])
    .env("RUSTC_BOOTSTRAP", "1")
    .output()?;
  let arch = if output.status.success() {
    let target_spec: TargetSpec = serde_json::from_reader(Cursor::new(output.stdout))?;
    target_spec
      .llvm_target
      .split('-')
      .next()
      .unwrap()
      .to_string()
  } else {
    super::common::print_info(&format!(
      "failed to determine target arch using rustc, error: `{}`. The fallback is the architecture of the machine that compiled this crate.",
      String::from_utf8_lossy(&output.stderr),
    ))?;
    if cfg!(target_arch = "x86") {
      "i686".into()
    } else if cfg!(target_arch = "x86_64") {
      "x86_64".into()
    } else if cfg!(target_arch = "arm") {
      "armv7".into()
    } else if cfg!(target_arch = "aarch64") {
      "aarch64".into()
    } else {
      return Err(crate::Error::ArchError(String::from(
        "Unable to determine target-architecture",
      )));
    }
  };

  let os = if cfg!(target_os = "linux") {
    "unknown-linux"
  } else if cfg!(target_os = "macos") {
    "apple-darwin"
  } else if cfg!(target_os = "windows") {
    "pc-windows"
  } else if cfg!(target_os = "freebsd") {
    "unknown-freebsd"
  } else {
    return Err(crate::Error::ArchError(String::from(
      "Unable to determine target-os",
    )));
  };

  let os = if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
    String::from(os)
  } else {
    let env = if cfg!(target_env = "gnu") {
      "gnu"
    } else if cfg!(target_env = "musl") {
      "musl"
    } else if cfg!(target_env = "msvc") {
      "msvc"
    } else {
      return Err(crate::Error::ArchError(String::from(
        "Unable to determine target-environment",
      )));
    };

    format!("{}-{}", os, env)
  };

  Ok(format!("{}-{}", arch, os))
}
