// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::OsStr, path::PathBuf, process::Command};

use crate::{Error, Result};
use rand::distr::{Alphanumeric, SampleString};

pub struct ProvisioningProfile {
  path: PathBuf,
}

/// Information read from a provisioning profile.
#[derive(Debug, Clone)]
pub struct ProvisioningProfileInfo {
  pub uuid: String,
  pub name: String,
  pub team_identifiers: Vec<String>,
  /// The `application-identifier` entitlement, `<TEAM_ID>.<bundle identifier>`.
  /// The bundle identifier part might be a wildcard (`*`).
  pub application_identifier: Option<String>,
  pub platforms: Vec<String>,
  pub expiration_date: Option<std::time::SystemTime>,
  /// Whether the profile has the `ProvisionsAllDevices` flag (enterprise distribution).
  pub provisions_all_devices: bool,
  /// Whether the profile lists devices it is restricted to (development and ad hoc profiles).
  pub has_provisioned_devices: bool,
  /// Whether the profile has the `get-task-allow` entitlement (development profiles).
  pub get_task_allow: bool,
}

impl ProvisioningProfileInfo {
  /// Reads the profile from the contents of a `.mobileprovision` or `.provisionprofile` file.
  ///
  /// Profiles are CMS signed messages containing a property list,
  /// which is read directly so this does not depend on the `security` CLI.
  pub fn parse(data: &[u8]) -> Result<Self> {
    let start = find(data, b"<?xml").ok_or(Error::FailedToDecodeProvisioningProfile)?;
    let end = find(&data[start..], b"</plist>")
      .map(|end| start + end + b"</plist>".len())
      .ok_or(Error::FailedToDecodeProvisioningProfile)?;

    let plist = plist::from_bytes::<plist::Dictionary>(&data[start..end])
      .map_err(|_| Error::FailedToDecodeProvisioningProfile)?;

    let string = |key: &str| {
      plist
        .get(key)
        .and_then(|v| v.as_string())
        .map(ToString::to_string)
    };
    let strings = |key: &str| {
      plist
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
          a.iter()
            .filter_map(|v| v.as_string().map(ToString::to_string))
            .collect()
        })
        .unwrap_or_default()
    };
    let entitlements = plist.get("Entitlements").and_then(|e| e.as_dictionary());

    Ok(Self {
      uuid: string("UUID").ok_or(Error::FailedToFindProvisioningProfileUuid)?,
      name: string("Name").unwrap_or_default(),
      team_identifiers: strings("TeamIdentifier"),
      application_identifier: entitlements
        .and_then(|e| {
          e.get("application-identifier")
            .or_else(|| e.get("com.apple.application-identifier"))
        })
        .and_then(|v| v.as_string())
        .map(ToString::to_string),
      platforms: strings("Platform"),
      expiration_date: plist
        .get("ExpirationDate")
        .and_then(|v| v.as_date())
        .map(Into::into),
      provisions_all_devices: plist
        .get("ProvisionsAllDevices")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false),
      has_provisioned_devices: plist.contains_key("ProvisionedDevices"),
      get_task_allow: entitlements
        .and_then(|e| e.get("get-task-allow"))
        .and_then(|v| v.as_boolean())
        .unwrap_or(false),
    })
  }

  /// The bundle identifier part of the application identifier.
  pub fn bundle_identifier(&self) -> Option<&str> {
    self
      .application_identifier
      .as_deref()
      .and_then(|id| id.split_once('.'))
      .map(|(_, bundle_id)| bundle_id)
  }

  /// Whether this profile can be used to sign an app with the given bundle identifier.
  pub fn matches_bundle_identifier(&self, bundle_identifier: &str) -> bool {
    match self.bundle_identifier() {
      Some(pattern) => match pattern.strip_suffix('*') {
        Some(prefix) => bundle_identifier.starts_with(prefix),
        None => pattern == bundle_identifier,
      },
      None => false,
    }
  }

  pub fn is_expired(&self) -> bool {
    self
      .expiration_date
      .is_some_and(|date| date < std::time::SystemTime::now())
  }

  /// Short description of the distribution method this profile is meant for.
  pub fn kind(&self) -> &'static str {
    if self.provisions_all_devices {
      "enterprise"
    } else if self.has_provisioned_devices {
      if self.get_task_allow {
        "development"
      } else {
        "ad hoc"
      }
    } else {
      "App Store"
    }
  }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack
    .windows(needle.len())
    .position(|window| window == needle)
}

impl ProvisioningProfile {
  pub fn from_base64(base64: &OsStr) -> Result<Self> {
    let home_dir = dirs::home_dir().ok_or(Error::ResolveHomeDir)?;
    let provisioning_profiles_folder = home_dir
      .join("Library")
      .join("MobileDevice")
      .join("Provisioning Profiles");
    std::fs::create_dir_all(&provisioning_profiles_folder).map_err(|error| Error::Fs {
      context: "failed to create provisioning profiles folder",
      path: provisioning_profiles_folder.clone(),
      error,
    })?;

    let provisioning_profile_path = provisioning_profiles_folder.join(format!(
      "{}.mobileprovision",
      Alphanumeric.sample_string(&mut rand::rng(), 16)
    ));
    super::decode_base64(base64, &provisioning_profile_path)?;

    Ok(Self {
      path: provisioning_profile_path,
    })
  }

  pub fn uuid(&self) -> Result<String> {
    let output = Command::new("security")
      .args(["cms", "-D", "-i"])
      .arg(&self.path)
      .output()
      .map_err(|error| Error::CommandFailed {
        command: "security cms -D -i".to_string(),
        error,
      })?;

    if !output.status.success() {
      return Err(Error::FailedToDecodeProvisioningProfile);
    }

    let plist =
      plist::from_bytes::<plist::Dictionary>(&output.stdout).map_err(|error| Error::Plist {
        context: "failed to parse provisioning profile as plist",
        path: self.path.clone(),
        error,
      })?;

    plist
      .get("UUID")
      .and_then(|v| v.as_string().map(ToString::to_string))
      .ok_or(Error::FailedToFindProvisioningProfileUuid)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn profile(application_identifier: &str, extra: &str) -> Vec<u8> {
    let plist = format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>UUID</key><string>1234-5678</string>
  <key>Name</key><string>Test Profile</string>
  <key>TeamIdentifier</key><array><string>TEAMID1234</string></array>
  <key>Platform</key><array><string>iOS</string></array>
  <key>ExpirationDate</key><date>2099-01-01T00:00:00Z</date>
  <key>Entitlements</key>
  <dict>
    <key>application-identifier</key><string>{application_identifier}</string>
  </dict>
  {extra}
</dict>
</plist>"#
    );
    // surround the plist with bytes as found in the CMS envelope
    let mut data = vec![0x30, 0x82, 0x01, 0x02];
    data.extend_from_slice(plist.as_bytes());
    data.extend_from_slice(&[0xa0, 0x82, 0x00]);
    data
  }

  #[test]
  fn parses_embedded_plist() {
    let info = ProvisioningProfileInfo::parse(&profile("TEAMID1234.com.tauri.app", "")).unwrap();
    assert_eq!(info.uuid, "1234-5678");
    assert_eq!(info.name, "Test Profile");
    assert_eq!(info.team_identifiers, vec!["TEAMID1234"]);
    assert_eq!(info.bundle_identifier(), Some("com.tauri.app"));
    assert!(info.matches_bundle_identifier("com.tauri.app"));
    assert!(!info.matches_bundle_identifier("com.tauri.other"));
    assert!(!info.is_expired());
    assert_eq!(info.kind(), "App Store");
  }

  #[test]
  fn matches_wildcard_and_detects_kind() {
    let info = ProvisioningProfileInfo::parse(&profile(
      "TEAMID1234.com.tauri.*",
      "<key>ProvisionedDevices</key><array><string>abc</string></array>",
    ))
    .unwrap();
    assert!(info.matches_bundle_identifier("com.tauri.app"));
    assert!(!info.matches_bundle_identifier("com.other.app"));
    assert_eq!(info.kind(), "ad hoc");
  }
}
