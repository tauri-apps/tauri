// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Decides whether Chromium's process sandbox has to be turned off.
//!
//! On every platform the answer normally comes straight from [`SandboxPolicy`], and the
//! answer is "keep it". Only Linux and the BSDs have a case where keeping it means the
//! application cannot start at all, and detecting that case is what the bulk of this
//! module is for.
//!
//! # The Linux case
//!
//! Chromium's zygote host picks, in order, the namespace sandbox when unprivileged user
//! namespaces work, then the root-owned setuid `chrome-sandbox` helper found next to the
//! executable (or at `CHROME_DEVEL_SANDBOX`). When neither is available it calls
//! `LOG(FATAL) << "No usable sandbox!"` and the process dies before a window is ever
//! shown.
//!
//! Tauri's deb and rpm bundlers install `chrome-sandbox` with mode 4755, so packaged
//! applications always have the helper. An AppImage cannot: the runtime mounts its
//! payload with `nosuid`, so a setuid binary inside it is inert. That leaves AppImages
//! relying on unprivileged user namespaces, which Ubuntu 23.10 and later restrict
//! through AppArmor — which is exactly the combination this module detects.
//!
//! Tauri's AppImage bundler does copy `chrome-sandbox` next to the main binary, so the
//! helper is *present* in every CEF AppImage and finding a file by that name proves
//! nothing; one inside an AppImage is never treated as available. Outside an AppImage
//! the file is stat'ed against the same conditions Chromium's zygote host applies —
//! owned by root, setuid, executable by others — because Chromium treats a helper that
//! fails them as a fatal error rather than falling back to another sandbox.
//!
//! # The Windows case
//!
//! **Windows currently runs unsandboxed, whatever the policy says.** Chromium's Windows
//! sandbox is brokered by the executable rather than by the library: CEF wants a
//! `sandbox_info` pointer from `cef_sandbox_info_create()` passed into both
//! `CefExecuteProcess` and `CefInitialize`, and when it gets a null one it sets
//! `CefSettings.no_sandbox` itself and appends `--no-sandbox`
//! (`libcef/browser/main_runner.cc`). This runtime passes null.
//!
//! Fixing that is a packaging change, not a code change: since Chromium M138 the sandbox
//! entry point can only be linked by a binary built with Chromium's own toolchain, so CEF
//! ships prebuilt `bootstrap.exe` / `bootstrapc.exe` hosts that load the application as a
//! DLL exporting `RunWinMain` or `RunConsoleMain` and hand it the pointer. A Tauri
//! application is built as an executable, so until it can be built and bundled as a
//! bootstrap-hosted DLL there is nothing to pass.
//!
//! Until then the honest thing is to say so: [`windows_sandbox_unavailable`] reports the
//! gap so [`SandboxPolicy::Auto`] logs it like any other lost sandbox, and
//! [`SandboxPolicy::Required`] fails loudly instead of quietly returning a promise the
//! platform cannot keep.
//!
//! # macOS
//!
//! macOS sandboxes through `libcef_sandbox.dylib`, which the helper process loads and
//! initializes before the framework, so there is nothing to probe: the policy decides on
//! its own and [`SandboxPolicy::Auto`] always keeps the sandbox.

use crate::runtime::SandboxPolicy;

/// Whether this platform can actually sandbox, given how the runtime initializes CEF.
///
/// Windows cannot yet: see the module docs. Kept as a function of a `cfg` rather than a
/// `cfg` at every use site so the decision table stays testable on one platform.
pub(crate) const fn windows_sandbox_unavailable() -> bool {
  cfg!(windows)
}

/// Why the sandbox is being turned off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SandboxDisableReason {
  /// The application asked for it through [`SandboxPolicy::Disabled`].
  Policy,
  /// AppImage, no setuid helper, and AppArmor restricts unprivileged user namespaces.
  AppImageUserNamespacesRestricted,
  /// AppImage, no setuid helper, and user namespaces are unavailable altogether.
  AppImageUserNamespacesUnavailable,
  /// Windows, where this runtime cannot supply CEF with a sandbox broker.
  WindowsBrokerUnavailable,
}

impl SandboxDisableReason {
  /// Message logged when the sandbox is dropped for this reason.
  pub(crate) fn message(self) -> &'static str {
    match self {
      Self::Policy => "the application set SandboxPolicy::Disabled",
      Self::AppImageUserNamespacesRestricted => {
        "running from an AppImage, which cannot ship the setuid chrome-sandbox helper, \
         and unprivileged user namespaces are restricted \
         (/proc/sys/kernel/apparmor_restrict_unprivileged_userns is 1)"
      }
      Self::AppImageUserNamespacesUnavailable => {
        "running from an AppImage, which cannot ship the setuid chrome-sandbox helper, \
         and unprivileged user namespaces are unavailable \
         (/proc/sys/user/max_user_namespaces is 0)"
      }
      Self::WindowsBrokerUnavailable => {
        "the Windows sandbox needs a broker this runtime cannot supply: CEF requires the \
         application to be hosted by its bootstrap executable as a DLL, and a Tauri \
         application is built as an executable"
      }
    }
  }
}

/// Whether `--no-sandbox` has to be appended, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SandboxDecision {
  /// Leave the sandbox alone.
  Keep,
  /// Append `--no-sandbox`, logging `reason`.
  Disable(SandboxDisableReason),
  /// [`SandboxPolicy::Required`] asked for a sandbox this platform cannot provide, so
  /// startup fails rather than silently running without one.
  Refuse(SandboxDisableReason),
}

/// Decides whether to disable the sandbox, from inputs the caller has already gathered.
///
/// Kept free of I/O so every combination can be unit tested.
///
/// `apparmor_restrict_unprivileged_userns` and `max_user_namespaces` are the values read
/// from `/proc/sys/kernel/apparmor_restrict_unprivileged_userns` and
/// `/proc/sys/user/max_user_namespaces`; [`None`] means the file could not be read,
/// which is treated as no evidence of a restriction rather than as a restriction.
///
/// `windows_broker_unavailable` is [`windows_sandbox_unavailable`]: on Windows CEF drops
/// the sandbox itself when the embedder hands it no broker, so the runtime cannot keep a
/// sandbox there however the policy is set.
pub(crate) fn sandbox_decision(
  policy: SandboxPolicy,
  windows_broker_unavailable: bool,
  running_from_appimage: bool,
  sandbox_helper_available: bool,
  apparmor_restrict_unprivileged_userns: Option<u64>,
  max_user_namespaces: Option<u64>,
) -> SandboxDecision {
  if let SandboxPolicy::Disabled = policy {
    return SandboxDecision::Disable(SandboxDisableReason::Policy);
  }

  // CEF flips `no_sandbox` on for us when it gets a null broker, so `Keep` here would be
  // a decision the platform overrules a moment later. Reporting it instead keeps the
  // rule that a lost sandbox is always named out loud.
  if windows_broker_unavailable {
    return match policy {
      SandboxPolicy::Required => {
        SandboxDecision::Refuse(SandboxDisableReason::WindowsBrokerUnavailable)
      }
      _ => SandboxDecision::Disable(SandboxDisableReason::WindowsBrokerUnavailable),
    };
  }

  match policy {
    SandboxPolicy::Disabled => SandboxDecision::Disable(SandboxDisableReason::Policy),
    SandboxPolicy::Required => SandboxDecision::Keep,
    SandboxPolicy::Auto => {
      // Everything but an AppImage can ship the setuid helper, so a missing sandbox
      // there is a packaging or system problem we should not paper over.
      if !running_from_appimage || sandbox_helper_available {
        return SandboxDecision::Keep;
      }

      if apparmor_restrict_unprivileged_userns == Some(1) {
        SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesRestricted)
      } else if max_user_namespaces == Some(0) {
        SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesUnavailable)
      } else {
        SandboxDecision::Keep
      }
    }
  }
}

/// Whether this process was launched with Chromium's `--no-sandbox` switch.
///
/// A child process inherits the switch from the browser process that spawned it, so this
/// is how a macOS helper learns that entering the sandbox would be wrong. It is read off
/// the real command line rather than off [`SandboxPolicy`] because a helper never sees
/// the `Cef` builder.
#[cfg(target_os = "macos")]
pub(crate) fn launched_without_sandbox() -> bool {
  std::env::args().any(|arg| arg == "--no-sandbox")
}

/// Whether a `chrome-sandbox` candidate passes the checks Chromium's zygote host makes
/// before it will use the helper, given the `st_uid` and `st_mode` a `stat` reported.
///
/// `ZygoteHostImpl::Init` requires the file to be owned by root, to carry the setuid bit
/// and to be executable by others; a file that is there but fails any of those aborts
/// with "The SUID sandbox helper binary was found, but is not configured correctly", so
/// a half-configured helper must not count as available.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub(crate) fn helper_stat_is_usable(uid: u32, mode: u32) -> bool {
  /// `S_ISUID`.
  const SETUID: u32 = 0o4000;
  /// `S_IXOTH`.
  const OTHER_EXECUTE: u32 = 0o0001;

  uid == 0 && mode & SETUID != 0 && mode & OTHER_EXECUTE != 0
}

/// Gathers the inputs [`sandbox_decision`] needs from the environment and the filesystem.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub(crate) fn resolve_sandbox_decision(policy: SandboxPolicy) -> SandboxDecision {
  let running_from_appimage = running_from_appimage();
  sandbox_decision(
    policy,
    windows_sandbox_unavailable(),
    running_from_appimage,
    sandbox_helper_available(running_from_appimage),
    read_sysctl("/proc/sys/kernel/apparmor_restrict_unprivileged_userns"),
    read_sysctl("/proc/sys/user/max_user_namespaces"),
  )
}

/// The policy's own answer, with nothing to probe: neither Windows nor macOS has an
/// equivalent of the AppImage case. macOS therefore keeps the sandbox under
/// [`SandboxPolicy::Auto`]; Windows cannot, for the reason in the module docs.
#[cfg(not(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
)))]
pub(crate) fn resolve_sandbox_decision(policy: SandboxPolicy) -> SandboxDecision {
  sandbox_decision(
    policy,
    windows_sandbox_unavailable(),
    false,
    false,
    None,
    None,
  )
}

/// AppImage runtimes export `APPIMAGE` with the path of the mounted image.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn running_from_appimage() -> bool {
  std::env::var_os("APPIMAGE").is_some_and(|path| !path.is_empty())
}

/// Whether Chromium can find *and use* the setuid `chrome-sandbox` helper.
///
/// The helper next to the executable is disregarded entirely when running from an
/// AppImage: the bundler always copies `chrome-sandbox` there, and the AppImage runtime
/// mounts the payload `nosuid`, so the setuid bit `stat` still reports has no effect.
///
/// `CHROME_DEVEL_SANDBOX` is somebody deliberately pointing at a helper outside the
/// application, so it is honoured on every layout, but it is stat'ed like any other
/// candidate.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn sandbox_helper_available(running_from_appimage: bool) -> bool {
  if let Some(path) = std::env::var_os("CHROME_DEVEL_SANDBOX").filter(|path| !path.is_empty()) {
    return helper_path_is_usable(std::path::Path::new(&path));
  }

  if running_from_appimage {
    return false;
  }

  std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|dir| dir.join("chrome-sandbox")))
    .is_some_and(|helper| helper_path_is_usable(&helper))
}

/// Stats `path` and hands what it reports to [`helper_stat_is_usable`].
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn helper_path_is_usable(path: &std::path::Path) -> bool {
  use std::os::unix::fs::MetadataExt;

  let Ok(metadata) = std::fs::metadata(path) else {
    return false;
  };

  let usable = helper_stat_is_usable(metadata.uid(), metadata.mode());
  if !usable {
    log::debug!(
      "ignoring the chrome-sandbox helper at {}: it is not a root-owned setuid binary executable by others",
      path.display()
    );
  }
  usable
}

/// Reads a numeric sysctl, returning [`None`] when it is missing or unparseable.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn read_sysctl(path: &str) -> Option<u64> {
  std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Shorthand for the `Auto` policy, which is the only one that inspects the system, on
  /// a platform whose sandbox broker works.
  fn auto(
    running_from_appimage: bool,
    sandbox_helper_available: bool,
    apparmor: Option<u64>,
    max_user_namespaces: Option<u64>,
  ) -> SandboxDecision {
    sandbox_decision(
      SandboxPolicy::Auto,
      false,
      running_from_appimage,
      sandbox_helper_available,
      apparmor,
      max_user_namespaces,
    )
  }

  #[test]
  fn explicit_policies_ignore_the_system() {
    for appimage in [false, true] {
      for helper in [false, true] {
        assert_eq!(
          sandbox_decision(
            SandboxPolicy::Disabled,
            false,
            appimage,
            helper,
            Some(1),
            Some(0)
          ),
          SandboxDecision::Disable(SandboxDisableReason::Policy)
        );
        assert_eq!(
          sandbox_decision(
            SandboxPolicy::Required,
            false,
            appimage,
            helper,
            Some(1),
            Some(0)
          ),
          SandboxDecision::Keep,
          "Required must keep the sandbox even when Chromium will abort"
        );
      }
    }
  }

  #[test]
  fn a_platform_without_a_broker_never_reports_a_sandbox_it_does_not_have() {
    // CEF sets `no_sandbox` itself when it gets a null broker, so claiming `Keep` here
    // would be a decision the platform overrules a moment later.
    assert_eq!(
      sandbox_decision(SandboxPolicy::Auto, true, false, false, None, None),
      SandboxDecision::Disable(SandboxDisableReason::WindowsBrokerUnavailable)
    );
  }

  #[test]
  fn required_refuses_to_start_where_the_sandbox_cannot_be_provided() {
    // The whole point of `Required` is that running unsandboxed is not an acceptable
    // outcome, so it must fail rather than come up without one.
    assert_eq!(
      sandbox_decision(SandboxPolicy::Required, true, false, false, None, None),
      SandboxDecision::Refuse(SandboxDisableReason::WindowsBrokerUnavailable)
    );
  }

  #[test]
  fn disabled_is_answered_before_the_platform_is_consulted() {
    // An application that asked for no sandbox is told what it asked for, not what the
    // platform could not give it.
    assert_eq!(
      sandbox_decision(SandboxPolicy::Disabled, true, false, false, None, None),
      SandboxDecision::Disable(SandboxDisableReason::Policy)
    );
  }

  #[test]
  fn auto_keeps_the_sandbox_outside_an_appimage() {
    // A deb or rpm install ships the setuid helper, and a system that lost it should
    // fail loudly rather than silently run unsandboxed.
    assert_eq!(auto(false, false, Some(1), Some(0)), SandboxDecision::Keep);
    assert_eq!(auto(false, true, None, None), SandboxDecision::Keep);
  }

  #[test]
  fn auto_keeps_the_sandbox_when_the_helper_is_available() {
    assert_eq!(auto(true, true, Some(1), Some(0)), SandboxDecision::Keep);
  }

  #[test]
  fn auto_keeps_the_sandbox_when_user_namespaces_work() {
    assert_eq!(
      auto(true, false, Some(0), Some(31231)),
      SandboxDecision::Keep
    );
  }

  #[test]
  fn auto_disables_for_an_appimage_restricted_by_apparmor() {
    assert_eq!(
      auto(true, false, Some(1), Some(31231)),
      SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesRestricted)
    );
  }

  #[test]
  fn auto_disables_for_an_appimage_without_user_namespaces() {
    assert_eq!(
      auto(true, false, Some(0), Some(0)),
      SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesUnavailable)
    );
    // The AppArmor sysctl only exists on kernels carrying that patch.
    assert_eq!(
      auto(true, false, None, Some(0)),
      SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesUnavailable)
    );
  }

  #[test]
  fn unreadable_sysctls_are_not_evidence_of_a_restriction() {
    // Nothing readable: assume namespaces work and let Chromium have the last word.
    assert_eq!(auto(true, false, None, None), SandboxDecision::Keep);
  }

  #[test]
  fn apparmor_restriction_is_reported_over_a_missing_namespace_quota() {
    // Both point the same way; the AppArmor one is the actionable message.
    assert_eq!(
      auto(true, false, Some(1), Some(0)),
      SandboxDecision::Disable(SandboxDisableReason::AppImageUserNamespacesRestricted)
    );
  }

  /// The platforms with nothing to probe answer from the policy alone, which is what
  /// [`resolve_sandbox_decision`] passes there. Asserted everywhere so the contract
  /// cannot drift on the platforms that do not compile that arm.
  #[test]
  fn nothing_to_probe_means_the_policy_decides() {
    assert_eq!(
      sandbox_decision(SandboxPolicy::Auto, false, false, false, None, None),
      SandboxDecision::Keep,
      "Auto must keep the sandbox where there is no AppImage case to escape"
    );
    assert_eq!(
      sandbox_decision(SandboxPolicy::Required, false, false, false, None, None),
      SandboxDecision::Keep
    );
    assert_eq!(
      sandbox_decision(SandboxPolicy::Disabled, false, false, false, None, None),
      SandboxDecision::Disable(SandboxDisableReason::Policy),
      "Disabled is the only way for macOS to lose the sandbox"
    );
  }

  /// The runtime hands CEF a null Windows sandbox broker, and CEF answers that by
  /// dropping the sandbox itself. Asserted on every platform so the constant cannot drift
  /// away from what `resolve_sandbox_decision` passes.
  #[test]
  fn the_windows_broker_is_reported_as_unavailable_only_on_windows() {
    assert_eq!(windows_sandbox_unavailable(), cfg!(windows));
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  #[test]
  fn a_correctly_installed_helper_is_usable() {
    // Mode 4755, which is what the deb and rpm bundlers install.
    assert!(helper_stat_is_usable(0, 0o104755));
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  #[test]
  fn a_helper_missing_any_of_chromiums_conditions_is_not_usable() {
    // Chromium aborts outright on a helper that fails these, so "present but wrong" has
    // to read as unavailable.
    assert!(
      !helper_stat_is_usable(1000, 0o104755),
      "a helper not owned by root cannot raise privileges"
    );
    assert!(
      !helper_stat_is_usable(0, 0o100755),
      "without the setuid bit the helper runs as the calling user"
    );
    assert!(
      !helper_stat_is_usable(0, 0o104750),
      "the helper has to be executable by others"
    );
    // What `fs::copy` produces in an AppDir: right name, none of the bits.
    assert!(!helper_stat_is_usable(1000, 0o100644));
  }
}
