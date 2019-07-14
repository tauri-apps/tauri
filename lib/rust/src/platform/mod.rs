pub mod error;
use error::*;

/// Try to determine the current target triple.
///
/// Returns a target triple (e.g. `x86_64-unknown-linux-gnu` or `i686-pc-windows-msvc`) or an
/// `Error::Config` if the current config cannot be determined or is not some combination of the
/// following values:
/// `linux, mac, windows` -- `i686, x86, armv7` -- `gnu, musl, msvc`
///
/// * Errors:
///     * Unexpected system config
pub fn target_triple() -> Result<String, Error> {
  let arch = if cfg!(target_arch = "x86") {
    "i686"
  } else if cfg!(target_arch = "x86_64") {
    "x86_64"
  } else if cfg!(target_arch = "arm") {
    "armv7"
  } else {
    bail!(Error::Arch, "Unable to determine target-architecture")
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
    bail!(Error::Target, "Unable to determine target-os");
  };

  let s;
  let os = if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
    os
  } else {
    let env = if cfg!(target_env = "gnu") {
      "gnu"
    } else if cfg!(target_env = "gnu") {
      "musl"
    } else if cfg!(target_env = "msvc") {
      "msvc"
    } else {
      bail!(Error::Abi, "Unable to determine target-environment")
    };
    s = format!("{}-{}", os, env);
    &s
  };

  Ok(format!("{}-{}", arch, os))
}
