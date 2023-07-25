// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Namespace lock file and utilities for the runtime authority.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{config::Namespace, plugin::ManifestMap};

/// Resolved data associated with a member.
#[derive(Deserialize, Serialize)]
pub struct MemberResolution {
  /// Member id.
  pub member: String,
  /// Resolved capabilities.
  pub capabilities: HashMap<String, ResolvedCapability>,
}

/// A resolved capability.
#[derive(Default, Deserialize, Serialize)]
pub struct ResolvedCapability {
  /// List of features enabled.
  pub features: Vec<String>,
}

/// Lock file of the namespaces configuration.
#[derive(Deserialize, Serialize)]
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
