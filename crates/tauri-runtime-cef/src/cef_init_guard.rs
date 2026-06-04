// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Graceful handling for a failed `cef::initialize()`.
//!
//! `cef::initialize()` returns `1` on success and `0` on failure. The runtime
//! used to `assert_eq!(cef::initialize(...), 1)`, which turned every failure
//! into a fatal `panic: assertion left == right` (Sentry **TAURI-RUST-F**,
//! thousands of events across Windows, Linux and macOS).
//!
//! `initialize()` can return `0` for several reasons, all of which mean the
//! same thing for us — CEF cannot run in this process and the app is therefore
//! non-functional:
//!   * a live prior instance still holds the CEF user-data-dir cache lock
//!     (sequential relaunch race — the dominant cause);
//!   * GPU / sandbox / permission failures;
//!   * a corrupt cache or read-only data dir.
//!
//! There is no in-process recovery: the lock / GPU / sandbox state will not
//! change within this process. The only correct action is to inform the user
//! and exit cleanly. Crucially this is the **single chokepoint** every
//! platform funnels through, so handling it here makes the panic structurally
//! impossible regardless of OS or root cause — no per-platform guard can.
//!
//! `fail_cef_init_and_exit` never returns and never panics: the user-facing
//! dialog is best-effort (wrapped in `catch_unwind`) and the process always
//! exits with code `0` afterwards, so a misbehaving dialog backend can never
//! re-introduce the very crash this module exists to prevent.

use std::path::Path;

const DIALOG_TITLE: &str = "OpenHuman couldn't start";
const DIALOG_BODY: &str = "OpenHuman's browser engine failed to start. \
This usually means another copy of OpenHuman is still running or shutting down.\n\n\
Please close any existing OpenHuman windows, wait a few seconds, and open it again.";

/// Called when `cef::initialize()` returns anything other than `1`.
///
/// Logs the failure with actionable context (the CEF return code and the
/// `cache_path` whose lock was likely contended), shows a best-effort native
/// dialog, and exits the process cleanly. Diverges (`-> !`); never panics.
///
/// The structured `log::error!` (return code + cache path + the stable
/// `TAURI-RUST-F` marker) preserves the Sentry-triage signal that a fatal
/// panic used to provide — without the crash.
pub(crate) fn fail_cef_init_and_exit(init_result: i32, cache_path: &Path) -> ! {
  log::error!(
    "[tauri-runtime-cef] cef::initialize returned {init_result} (expected 1) for cache_path={} — \
CEF cannot start in this process (cache locked by a live prior instance, or a GPU/sandbox/permission \
failure). Showing a notice and exiting cleanly instead of panicking (TAURI-RUST-F).",
    cache_path.display()
  );

  // Best-effort, never-panic: a dialog backend failure must not turn the
  // graceful exit back into a crash. Any unwind is swallowed; we exit(0)
  // regardless of whether the dialog actually displayed.
  let _ = std::panic::catch_unwind(show_blocking_notice);

  std::process::exit(0);
}

/// Show a modal "couldn't start" notice using whatever native dialog facility
/// the platform already links. Best-effort: returns quietly if it cannot show.
fn show_blocking_notice() {
  #[cfg(target_os = "macos")]
  macos::show(DIALOG_TITLE, DIALOG_BODY);

  #[cfg(windows)]
  windows_impl::show(DIALOG_TITLE, DIALOG_BODY);

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
  ))]
  gtk_impl::show(DIALOG_TITLE, DIALOG_BODY);
}

#[cfg(target_os = "macos")]
mod macos {
  use objc2::MainThreadMarker;
  use objc2_app_kit::{NSAlert, NSApplication};
  use objc2_foundation::NSString;

  pub(super) fn show(title: &str, body: &str) {
    // `init()` runs on the main thread; if for any reason we are not on it,
    // skip the dialog rather than risk undefined behaviour.
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    // `NSAlert::runModal` requires a shared application to exist. CEF init
    // failed before Tauri created one, so make sure it is present.
    let _app = NSApplication::sharedApplication(mtm);
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(body));
    alert.runModal();
  }
}

#[cfg(windows)]
mod windows_impl {
  use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
  use windows::core::HSTRING;

  pub(super) fn show(title: &str, body: &str) {
    let body = HSTRING::from(body);
    let title = HSTRING::from(title);
    // SAFETY: FFI call with valid, NUL-terminated wide strings owned for
    // the duration of the call; a null owner window is valid for a
    // standalone modal box.
    unsafe {
      MessageBoxW(None, &body, &title, MB_OK | MB_ICONERROR);
    }
  }
}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
mod gtk_impl {
  pub(super) fn show(title: &str, body: &str) {
    // GTK must be initialised before any widget is created. `gtk::init` is
    // idempotent, so calling it here (CEF init already failed, so GTK was
    // never taken) is safe even though the normal startup path also calls
    // it. If it fails (headless), skip the dialog.
    if gtk::init().is_err() {
      return;
    }
    let dialog = gtk::MessageDialog::new(
      None::<&gtk::Window>,
      gtk::DialogFlags::MODAL,
      gtk::MessageType::Error,
      gtk::ButtonsType::Ok,
      body,
    );
    dialog.set_title(title);
    // `run` spins its own nested main loop and blocks until the user
    // dismisses the dialog. Dropping the dialog afterwards unrefs/closes
    // the window; the process exits immediately after `show` returns.
    dialog.run();
  }
}
