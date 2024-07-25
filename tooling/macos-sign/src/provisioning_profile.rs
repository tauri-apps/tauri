// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::OsStr, path::PathBuf, process::Command};

use anyhow::{Context, Result};
use rand::distributions::{Alphanumeric, DistString};

pub struct ProvisioningProfile {
  path: PathBuf,
}

impl ProvisioningProfile {
  pub fn from_base64(base64: &OsStr) -> Result<Self> {
    let home_dir = dirs_next::home_dir().unwrap();
    let provisioning_profiles_folder = home_dir
      .join("Library")
      .join("MobileDevice")
      .join("Provisioning Profiles");
    std::fs::create_dir_all(&provisioning_profiles_folder).unwrap();

    let provisioning_profile_path = provisioning_profiles_folder.join(format!(
      "{}.mobileprovision",
      Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
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
      .output()?;

    if !output.status.success() {
      return Err(anyhow::anyhow!("failed to decode provisioning profile"));
    }

    let plist = plist::from_bytes::<plist::Dictionary>(&output.stdout)
      .context("failed to decode provisioning profile as plist")?;

    plist
      .get("UUID")
      .and_then(|v| v.as_string().map(ToString::to_string))
      .ok_or_else(|| anyhow::anyhow!("could not find provisioning profile UUID"))
  }
}
