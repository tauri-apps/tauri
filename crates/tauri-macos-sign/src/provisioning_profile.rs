// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::OsStr, path::PathBuf, process::Command};

use crate::{Error, Result};
use rand::distr::{Alphanumeric, SampleString};

pub struct ProvisioningProfile {
  path: PathBuf,
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
