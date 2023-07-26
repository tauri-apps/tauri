// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Namespace lock file and utilities for the runtime authority.

use serde::{Deserialize, Serialize};

use crate::{config::Namespace, plugin::ManifestMap};

/// Resolved data associated with a member.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MemberResolution {
  /// Member id.
  pub member: String,
  /// List of commands enabled.
  pub commands: Vec<String>,
}

/// Lock file of the namespaces configuration.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NamespaceLockFile {
  /// Lock file version.
  pub version: u8,
  /// Configured namespaces.
  pub namespaces: Vec<Namespace>,
  /// Collection of plugins and their manifests.
  pub plugins: ManifestMap,
  /// Resolved data.
  pub resolution: Vec<MemberResolution>,
}
