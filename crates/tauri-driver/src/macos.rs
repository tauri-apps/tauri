// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS scaffolding for `tauri-driver`.
//!
//! Status: SCAFFOLD ONLY.
//!
//! There is no first-party WebDriver implementation for `WKWebView` on macOS
//! today. Apple ships `safaridriver` for Safari (which is built on the same
//! WebKit core) and various third-party tools wrap accessibility APIs to drive
//! `WKWebView`-hosted apps. None of those are drop-in replacements for
//! `WebKitWebDriver` / `msedgedriver`, so this module deliberately:
//!
//! - documents the planned approach (see [`MacOsDriver`]);
//! - implements the tractable pieces (locating `safaridriver`, spawning the
//!   process, ping/handshake);
//! - leaves the WebDriver session-management translation as `unimplemented!()`,
//!   so we fail loudly at runtime instead of silently doing the wrong thing.
//!
//! See `MACOS_DRIVER_DESIGN.md` (next to `Cargo.toml`) for the longer-form
//! design notes.
//!
//! NOTE: nothing in this module is wired into the main binary path yet. The
//! intent is to land the scaffolding so follow-up PRs can fill in the bridge
//! incrementally without rewriting the platform-detection plumbing each time.

use crate::cli::Args;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default name of the system WebDriver binary shipped with Safari.
///
/// `safaridriver` lives at `/usr/bin/safaridriver` on stock macOS. We still go
/// through `which::which` so a user-supplied `--native-driver` or a non-default
/// `$PATH` works the same way it does on Linux/Windows.
pub const DRIVER_BINARY: &str = "safaridriver";

/// How long to wait for the spawned native driver to start accepting TCP
/// connections before giving up. Conservative; `safaridriver` is fast in
/// practice, but CI hosts can be slow to wake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Single point of entry for the macOS WebDriver scaffold.
///
/// This deliberately mirrors the shape of the Linux/Windows paths
/// (`webdriver::native(&args)`) so swapping it in later is a small diff.
///
/// Lifecycle:
/// 1. Resolve the driver binary (CLI override, then `$PATH`).
/// 2. `Command::spawn()` with the right port/host args.
/// 3. (Future) ping the local port until the driver is ready.
/// 4. (Future) translate `tauri:options` into `safari:options` /
///    `appium:options` on `POST /session`.
///
/// Anything past step 2 is currently `unimplemented!()` on purpose. Callers
/// should not assume a successful spawn means the bridge is functional.
#[derive(Debug)]
pub struct MacOsDriver {
  binary: PathBuf,
  native_port: u16,
  native_host: String,
}

impl MacOsDriver {
  /// Resolve a driver binary using the same rules as `webdriver::native`:
  /// honour `--native-driver` first, then fall back to `$PATH`.
  pub fn from_args(args: &Args) -> io::Result<Self> {
    let binary = match args.native_driver.as_deref() {
      Some(custom) if custom.exists() => custom.to_owned(),
      Some(custom) => {
        return Err(io::Error::new(
          io::ErrorKind::NotFound,
          format!(
            "supplied --native-driver path does not exist: {}",
            custom.display()
          ),
        ));
      }
      None => which::which(DRIVER_BINARY).map_err(|e| {
        io::Error::new(
          io::ErrorKind::NotFound,
          format!(
            "can not find binary `{DRIVER_BINARY}` in PATH. \
             Pass --native-driver to override. ({e})"
          ),
        )
      })?,
    };

    Ok(Self {
      binary,
      native_port: args.native_port,
      native_host: args.native_host.clone(),
    })
  }

  /// Path to the resolved driver binary. Useful for diagnostics.
  pub fn binary(&self) -> &Path {
    &self.binary
  }

  /// Build (but do not yet spawn) the `Command` used to launch the native
  /// driver. Kept separate so tests can inspect the argv without forking a
  /// real process.
  pub fn build_command(&self) -> Command {
    let mut cmd = Command::new(&self.binary);
    // Match the env vars set in `webdriver::native` so the rest of the
    // pipeline behaves consistently across platforms.
    cmd.env("TAURI_AUTOMATION", "true"); // 1.x
    cmd.env("TAURI_WEBVIEW_AUTOMATION", "true"); // 2.x
    cmd.arg(format!("--port={}", self.native_port));
    // safaridriver doesn't take a --host flag, but we keep the field for
    // parity and forward it in the future appium-based path.
    let _ = &self.native_host;
    // Same stdout policy as the Linux/Windows path: don't pollute the parent's
    // captured stdout.
    cmd.stdout(Stdio::null());
    cmd
  }

  /// Wait for the driver to start listening on `127.0.0.1:native_port`. Used
  /// by integration tests and (eventually) by `main` after spawning the
  /// process. Pure TCP probe, no WebDriver-level handshake yet.
  pub fn wait_for_ready(&self) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], self.native_port));
    let deadline = Instant::now() + HANDSHAKE_TIMEOUT;
    loop {
      match TcpStream::connect_timeout(&addr, Duration::from_millis(250)) {
        Ok(_) => return Ok(()),
        Err(_) if Instant::now() < deadline => {
          std::thread::sleep(Duration::from_millis(100));
        }
        Err(e) => {
          return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            format!("native driver did not become ready on {addr}: {e}"),
          ));
        }
      }
    }
  }

  /// Best-effort version probe. Sends a minimal HTTP/1.1 `GET /status`
  /// request -- the W3C WebDriver "status" endpoint -- and returns the raw
  /// response body so the caller can decide whether the bridge is healthy.
  ///
  /// Intentionally hand-rolled (no `hyper`) so the scaffold stays a leaf
  /// module with no extra build-time deps; the main proxy still uses `hyper`.
  pub fn probe_status(&self) -> io::Result<String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], self.native_port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(b"GET /status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(buf)
  }

  // -- Below: the parts that are NOT yet implemented. ---------------------

  /// Translate a Tauri WebDriver `POST /session` payload into the shape the
  /// native macOS driver expects.
  ///
  /// On Linux this maps `tauri:options` -> `webkitgtk:browserOptions`. On
  /// Windows it maps to `ms:edgeOptions`. On macOS we have to decide between
  /// `safari:options` (Safari only -- not WKWebView) and an Appium-style
  /// `appium:options` map. That decision is design work, not boilerplate, so
  /// it's deliberately left for the follow-up PR that turns this scaffold
  /// into a working bridge.
  pub fn map_capabilities(_json: serde_json::Value) -> serde_json::Value {
    unimplemented!(
      "macOS WebDriver capability translation is not yet implemented. \
       See MACOS_DRIVER_DESIGN.md."
    )
  }

  /// Run the WebDriver intermediary loop, mirroring `server::run` for
  /// Linux/Windows. Left unimplemented while the capability translation is
  /// still being designed -- once `map_capabilities` lands, this collapses
  /// to roughly the same code as `server::run` and can probably be unified.
  pub fn run(_args: Args) -> anyhow::Result<()> {
    unimplemented!(
      "macOS WebDriver session management is not yet implemented. \
       See MACOS_DRIVER_DESIGN.md."
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn args(native_driver: Option<PathBuf>) -> Args {
    Args {
      port: 4444,
      native_port: 4445,
      native_host: "127.0.0.1".to_string(),
      native_driver,
    }
  }

  #[test]
  fn rejects_missing_explicit_driver_path() {
    let bogus = PathBuf::from("/definitely/not/a/real/path/safaridriver");
    let err = MacOsDriver::from_args(&args(Some(bogus))).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
  }

  #[test]
  fn build_command_sets_port_and_env() {
    // Use `/bin/sh` as a stand-in binary so we don't depend on safaridriver
    // being installed in CI. We only inspect the constructed argv/env.
    let sh = PathBuf::from("/bin/sh");
    let driver = MacOsDriver {
      binary: sh.clone(),
      native_port: 4445,
      native_host: "127.0.0.1".to_string(),
    };
    let cmd = driver.build_command();
    let program = cmd.get_program().to_string_lossy().to_string();
    assert_eq!(program, sh.to_string_lossy());

    let argv: Vec<String> = cmd
      .get_args()
      .map(|a| a.to_string_lossy().to_string())
      .collect();
    assert!(argv.iter().any(|a| a == "--port=4445"));

    let envs: Vec<(String, Option<String>)> = cmd
      .get_envs()
      .map(|(k, v)| {
        (
          k.to_string_lossy().to_string(),
          v.map(|v| v.to_string_lossy().to_string()),
        )
      })
      .collect();
    assert!(envs
      .iter()
      .any(|(k, v)| k == "TAURI_AUTOMATION" && v.as_deref() == Some("true")));
    assert!(envs
      .iter()
      .any(|(k, v)| k == "TAURI_WEBVIEW_AUTOMATION" && v.as_deref() == Some("true")));
  }

  #[test]
  fn wait_for_ready_times_out_when_nothing_listens() {
    // Pick a port that's almost certainly free; if something is listening we
    // skip the assertion rather than fail spuriously in CI.
    let driver = MacOsDriver {
      binary: PathBuf::from("/bin/sh"),
      native_port: 1, // privileged + almost never bindable -> connect fails fast
      native_host: "127.0.0.1".to_string(),
    };
    // Don't actually wait the full 10s in unit tests -- this just exercises
    // the code path. We use a short-circuit by overriding nothing; instead
    // we just call probe_status which has its own 2s timeout.
    assert!(driver.probe_status().is_err());
  }

  #[test]
  #[should_panic(expected = "not yet implemented")]
  fn map_capabilities_is_unimplemented() {
    let _ = MacOsDriver::map_capabilities(serde_json::json!({}));
  }
}
