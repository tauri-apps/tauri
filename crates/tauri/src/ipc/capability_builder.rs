// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;
use tauri_utils::{
  acl::{
    capability::{Capability, CapabilityFile, PermissionEntry},
    Scopes,
  },
  platform::Target,
};

/// A capability that can be added at runtime.
pub trait RuntimeCapability {
  /// Creates the capability file.
  fn build(self) -> CapabilityFile;
}

impl<T: AsRef<str>> RuntimeCapability for T {
  fn build(self) -> CapabilityFile {
    self.as_ref().parse().expect("invalid capability")
  }
}

/// A builder for a [`Capability`].
pub struct CapabilityBuilder(Capability);

impl CapabilityBuilder {
  /// Creates a new capability builder with a unique identifier.
  pub fn new(identifier: impl Into<String>) -> Self {
    Self(Capability {
      identifier: identifier.into(),
      description: "".into(),
      remote: None,
      local: true,
      windows: Vec::new(),
      webviews: Vec::new(),
      permissions: Vec::new(),
      platforms: None,
    })
  }

  /// Allows this capability to be used by a remote URL.
  pub fn remote(mut self, url: String) -> Self {
    self
      .0
      .remote
      .get_or_insert_with(Default::default)
      .urls
      .push(url);
    self
  }

  /// Whether this capability is applied on local app URLs or not. Defaults to `true`.
  pub fn local(mut self, local: bool) -> Self {
    self.0.local = local;
    self
  }

  /// Link this capability to the given window label.
  pub fn window(mut self, window: impl Into<String>) -> Self {
    self.0.windows.push(window.into());
    self
  }

  /// Link this capability to the a list of window labels.
  pub fn windows(mut self, windows: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self.0.windows.extend(windows.into_iter().map(|w| w.into()));
    self
  }

  /// Link this capability to the given webview label.
  pub fn webview(mut self, webview: impl Into<String>) -> Self {
    self.0.webviews.push(webview.into());
    self
  }

  /// Link this capability to the a list of window labels.
  pub fn webviews(mut self, webviews: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self
      .0
      .webviews
      .extend(webviews.into_iter().map(|w| w.into()));
    self
  }

  /// Add a new permission to this capability.
  pub fn permission(mut self, permission: impl Into<String>) -> Self {
    let permission = permission.into();
    self.0.permissions.push(PermissionEntry::PermissionRef(
      permission
        .clone()
        .try_into()
        .unwrap_or_else(|_| panic!("invalid permission identifier '{permission}'")),
    ));
    self
  }

  /// Add a new scoped permission to this capability.
  pub fn permission_scoped<T: Serialize>(
    mut self,
    permission: impl Into<String>,
    allowed: Vec<T>,
    denied: Vec<T>,
  ) -> Self {
    let permission = permission.into();
    let identifier = permission
      .clone()
      .try_into()
      .unwrap_or_else(|_| panic!("invalid permission identifier '{permission}'"));

    let allowed_scope = allowed
      .into_iter()
      .map(|a| {
        serde_json::to_value(a)
          .expect("failed to serialize scope")
          .into()
      })
      .collect();
    let denied_scope = denied
      .into_iter()
      .map(|a| {
        serde_json::to_value(a)
          .expect("failed to serialize scope")
          .into()
      })
      .collect();
    let scope = Scopes {
      allow: Some(allowed_scope),
      deny: Some(denied_scope),
    };

    self
      .0
      .permissions
      .push(PermissionEntry::ExtendedPermission { identifier, scope });
    self
  }

  /// Adds a target platform for this capability.
  ///
  /// By default all platforms are applied.
  pub fn platform(mut self, platform: Target) -> Self {
    self
      .0
      .platforms
      .get_or_insert_with(Default::default)
      .push(platform);
    self
  }

  /// Adds target platforms for this capability.
  ///
  /// By default all platforms are applied.
  pub fn platforms(mut self, platforms: impl IntoIterator<Item = Target>) -> Self {
    self
      .0
      .platforms
      .get_or_insert_with(Default::default)
      .extend(platforms);
    self
  }
}

impl RuntimeCapability for CapabilityBuilder {
  fn build(self) -> CapabilityFile {
    CapabilityFile::Capability(self.0)
  }
}
