// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::utils::CommandExt;
use std::process::Command;

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[derive(Debug, PartialEq, Eq)]
struct RustCfg {
  target_arch: Option<String>,
}

fn parse_rust_cfg(cfg: String) -> RustCfg {
  let target_line = "target_arch=\"";
  let mut target_arch = None;
  for line in cfg.split('\n') {
    if line.starts_with(target_line) {
      let len = target_line.len();
      let arch = line.chars().skip(len).take(line.len() - len - 1).collect();
      target_arch.replace(arch);
    }
  }
  RustCfg { target_arch }
}

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
  let arch_res = Command::new("rustc").args(["--print", "cfg"]).output_ok();

  let arch = match arch_res {
    Ok(output) => parse_rust_cfg(String::from_utf8_lossy(&output.stdout).into())
      .target_arch
      .expect("could not find `target_arch` when running `rustc --print cfg`."),
    Err(err) => {
      log::warn!(
      "failed to determine target arch using rustc, error: `{}`. The fallback is the architecture of the machine that compiled this crate.",
      err,
    );
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

    format!("{os}-{env}")
  };

  Ok(format!("{arch}-{os}"))
}

#[cfg(test)]
mod tests {
  use super::RustCfg;

  #[test]
  fn parse_rust_cfg() {
    assert_eq!(
      super::parse_rust_cfg("target_arch".into()),
      RustCfg { target_arch: None }
    );

    assert_eq!(
      super::parse_rust_cfg(
        r#"debug_assertions
target_arch="aarch64"
target_endian="little"
target_env=""
target_family="unix"
target_os="macos"
target_pointer_width="64"
target_vendor="apple"
unix"#
          .into()
      ),
      RustCfg {
        target_arch: Some("aarch64".into())
      }
    );
  }
}
