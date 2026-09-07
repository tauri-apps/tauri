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

/// Gathers the inputs [`sandbox_decision`] needs from the environment and the filesystem.
pub(crate) fn resolve_sandbox_decision(policy: LinuxSandboxPolicy) -> SandboxDecision {
  sandbox_decision(
    policy,
    running_from_appimage(),
    sandbox_helper_available(),
    read_sysctl("/proc/sys/kernel/apparmor_restrict_unprivileged_userns"),
    read_sysctl("/proc/sys/user/max_user_namespaces"),
  )
}

/// AppImage runtimes export `APPIMAGE` with the path of the mounted image.
fn running_from_appimage() -> bool {
  std::env::var_os("APPIMAGE").is_some_and(|path| !path.is_empty())
}

/// Whether Chromium can find the setuid `chrome-sandbox` helper.
fn sandbox_helper_available() -> bool {
  if std::env::var_os("CHROME_DEVEL_SANDBOX").is_some_and(|path| !path.is_empty()) {
    return true;
  }

  let Ok(exe) = std::env::current_exe() else {
    return false;
  };
  exe
    .parent()
    .map(|dir| dir.join("chrome-sandbox"))
    .is_some_and(|helper| helper.exists())
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
}
