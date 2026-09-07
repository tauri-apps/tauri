// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Decides whether Chromium's Linux sandbox has to be turned off for the process to
//! start at all.
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
//! helper is *present* in every CEF AppImage. Merely finding a file by that name
//! therefore proves nothing, and this module never treats one inside an AppImage as
//! available. Outside an AppImage the file is stat'ed against the same conditions
//! Chromium's zygote host applies — owned by root, setuid, executable by others — which
//! is also why a half-configured helper must not count as available: Chromium treats one
//! that fails those checks as a fatal error rather than falling back to another sandbox.

/// What to do with Chromium's sandbox on Linux and the BSDs.
///
/// Defaults to [`LinuxSandboxPolicy::Auto`], which keeps the sandbox on unless the
/// application is running from an AppImage on a system that offers no way to sandbox at
/// all — where the alternative is not an unsandboxed application but no application,
/// since Chromium aborts with "No usable sandbox!".
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LinuxSandboxPolicy {
  /// Keep the sandbox, except when the application runs from an AppImage and the system
  /// has neither the setuid `chrome-sandbox` helper nor usable unprivileged user
  /// namespaces. A warning naming the reason is logged whenever the sandbox is dropped.
  #[default]
  Auto,
  /// Never pass `--no-sandbox`, even when that means Chromium aborts at startup.
  ///
  /// Pick this when running unsandboxed is not an acceptable outcome and a hard failure
  /// is preferable — the user can then install the setuid helper, point
  /// `CHROME_DEVEL_SANDBOX` at one, or re-enable unprivileged user namespaces.
  Required,
  /// Always pass `--no-sandbox`.
  ///
  /// Every renderer then runs with the full privileges of the user, so a compromised
  /// renderer is a compromised account. Useful for containers and CI images that cannot
  /// provide a sandbox, not for shipped applications.
  Disabled,
}

/// Why the sandbox is being turned off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SandboxDisableReason {
  /// The application asked for it through [`LinuxSandboxPolicy::Disabled`].
  Policy,
  /// AppImage, no setuid helper, and AppArmor restricts unprivileged user namespaces.
  AppImageUserNamespacesRestricted,
  /// AppImage, no setuid helper, and user namespaces are unavailable altogether.
  AppImageUserNamespacesUnavailable,
}

impl SandboxDisableReason {
  /// Message logged when the sandbox is dropped for this reason.
  pub(crate) fn message(self) -> &'static str {
    match self {
      Self::Policy => "the application set LinuxSandboxPolicy::Disabled",
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
}

/// Decides whether to disable the sandbox, from inputs the caller has already gathered.
///
/// Kept free of I/O so every combination can be unit tested.
///
/// `apparmor_restrict_unprivileged_userns` and `max_user_namespaces` are the values read
/// from `/proc/sys/kernel/apparmor_restrict_unprivileged_userns` and
/// `/proc/sys/user/max_user_namespaces`; [`None`] means the file could not be read,
/// which is treated as no evidence of a restriction rather than as a restriction.
pub(crate) fn sandbox_decision(
  policy: LinuxSandboxPolicy,
  running_from_appimage: bool,
  sandbox_helper_available: bool,
  apparmor_restrict_unprivileged_userns: Option<u64>,
  max_user_namespaces: Option<u64>,
) -> SandboxDecision {
  match policy {
    LinuxSandboxPolicy::Disabled => SandboxDecision::Disable(SandboxDisableReason::Policy),
    LinuxSandboxPolicy::Required => SandboxDecision::Keep,
    LinuxSandboxPolicy::Auto => {
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

/// Whether a `chrome-sandbox` candidate passes the checks Chromium's zygote host makes
/// before it will use the helper, given the `st_uid` and `st_mode` a `stat` reported.
///
/// `ZygoteHostImpl::Init` requires the file to be owned by root, to carry the setuid bit
/// and to be executable by others; a file that is there but fails any of those makes
/// Chromium abort with "The SUID sandbox helper binary was found, but is not configured
/// correctly", so a half-configured helper is worse than none and must not count as
/// available.
pub(crate) fn helper_stat_is_usable(uid: u32, mode: u32) -> bool {
  /// `S_ISUID`.
  const SETUID: u32 = 0o4000;
  /// `S_IXOTH`.
  const OTHER_EXECUTE: u32 = 0o0001;

  uid == 0 && mode & SETUID != 0 && mode & OTHER_EXECUTE != 0
}

/// Gathers the inputs [`sandbox_decision`] needs from the environment and the filesystem.
pub(crate) fn resolve_sandbox_decision(policy: LinuxSandboxPolicy) -> SandboxDecision {
  let running_from_appimage = running_from_appimage();
  sandbox_decision(
    policy,
    running_from_appimage,
    sandbox_helper_available(running_from_appimage),
    read_sysctl("/proc/sys/kernel/apparmor_restrict_unprivileged_userns"),
    read_sysctl("/proc/sys/user/max_user_namespaces"),
  )
}

/// AppImage runtimes export `APPIMAGE` with the path of the mounted image.
fn running_from_appimage() -> bool {
  std::env::var_os("APPIMAGE").is_some_and(|path| !path.is_empty())
}

/// Whether Chromium can find *and use* the setuid `chrome-sandbox` helper.
///
/// The helper next to the executable is disregarded entirely when running from an
/// AppImage. Tauri's AppImage bundler copies `chrome-sandbox` into the same directory as
/// the main binary, so the file is always there, and the AppImage runtime mounts the
/// payload `nosuid`, so its setuid bit — which `stat` still reports — has no effect when
/// Chromium tries to execute it.
///
/// `CHROME_DEVEL_SANDBOX` is somebody deliberately pointing at a helper outside the
/// application, so it is honoured on every layout, but it is stat'ed like any other
/// candidate: the variable merely being set says nothing about the file it names.
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
fn read_sysctl(path: &str) -> Option<u64> {
  std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Shorthand for the `Auto` policy, which is the only one that inspects the system.
  fn auto(
    running_from_appimage: bool,
    sandbox_helper_available: bool,
    apparmor: Option<u64>,
    max_user_namespaces: Option<u64>,
  ) -> SandboxDecision {
    sandbox_decision(
      LinuxSandboxPolicy::Auto,
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
            LinuxSandboxPolicy::Disabled,
            appimage,
            helper,
            Some(1),
            Some(0)
          ),
          SandboxDecision::Disable(SandboxDisableReason::Policy)
        );
        assert_eq!(
          sandbox_decision(
            LinuxSandboxPolicy::Required,
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

  #[test]
  fn a_correctly_installed_helper_is_usable() {
    // Mode 4755, which is what the deb and rpm bundlers install.
    assert!(helper_stat_is_usable(0, 0o104755));
  }

  #[test]
  fn a_helper_missing_any_of_chromiums_conditions_is_not_usable() {
    // Chromium aborts outright on a helper that fails these, so "present but wrong" has
    // to read as unavailable, not as a sandbox we can rely on.
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
