// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! End-user abstraction for selecting permissions a window has access to.

use crate::{acl::Identifier, platform::Target};
use serde::{Deserialize, Serialize};

use super::Scopes;

/// An entry for a permission value in a [`Capability`] can be either a raw permission [`Identifier`]
/// or an object that references a permission and extends its scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PermissionEntry {
  /// Reference a permission or permission set by identifier.
  PermissionRef(Identifier),
  /// Reference a permission or permission set by identifier and extends its scope.
  ExtendedPermission {
    /// Identifier of the permission or permission set.
    identifier: Identifier,
    /// Scope to append to the existing permission scope.
    #[serde(default, flatten)]
    scope: Scopes,
  },
}

impl PermissionEntry {
  /// The identifier of the permission referenced in this entry.
  pub fn identifier(&self) -> &Identifier {
    match self {
      Self::PermissionRef(identifier) => identifier,
      Self::ExtendedPermission {
        identifier,
        scope: _,
      } => identifier,
    }
  }
}

/// a grouping and boundary mechanism developers can use to separate windows or plugins functionality from each other at runtime.
///
/// If a window is not matching any capability then it has no access to the IPC layer at all.
///
/// This can be done to create trust groups and reduce impact of vulnerabilities in certain plugins or windows.
/// Windows can be added to a capability by exact name or glob patterns like *, admin-* or main-window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Capability {
  /// Identifier of the capability.
  pub identifier: String,
  /// Description of the capability.
  #[serde(default)]
  pub description: String,
  /// Execution context of the capability.
  ///
  /// At runtime, Tauri filters the IPC command together with the context to determine whether it is allowed or not and its scope.
  #[serde(default)]
  pub context: CapabilityContext,
  /// List of windows that uses this capability. Can be a glob pattern.
  ///
  /// On multiwebview windows, prefer [`Self::webviews`] for a fine grained access control.
  pub windows: Vec<String>,
  /// List of webviews that uses this capability. Can be a glob pattern.
  ///
  /// This is only required when using on multiwebview contexts, by default
  /// all child webviews of a window that matches [`Self::windows`] are linked.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub webviews: Vec<String>,
  /// List of permissions attached to this capability. Must include the plugin name as prefix in the form of `${plugin-name}:${permission-name}`.
  pub permissions: Vec<PermissionEntry>,
  /// Target platforms this capability applies. By default all platforms applies.
  #[serde(default = "default_platforms")]
  pub platforms: Vec<Target>,
}

fn default_platforms() -> Vec<Target> {
  vec![
    Target::Linux,
    Target::MacOS,
    Target::Windows,
    Target::Android,
    Target::Ios,
  ]
}

/// Context of the capability.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum CapabilityContext {
  /// Capability refers to local URL usage.
  #[default]
  Local,
  /// Capability refers to remote usage.
  Remote {
    /// Remote domains this capability refers to. Can use glob patterns.
    domains: Vec<String>,
  },
}

#[cfg(feature = "build")]
mod build {
  use std::convert::identity;

  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  use super::*;
  use crate::{literal_struct, tokens::*};

  impl ToTokens for CapabilityContext {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::capability::CapabilityContext };

      tokens.append_all(match self {
        Self::Remote { domains } => {
          let domains = vec_lit(domains, str_lit);
          quote! { #prefix::Remote { domains: #domains } }
        }
        Self::Local => {
          quote! { #prefix::Local }
        }
      });
    }
  }

  impl ToTokens for PermissionEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::capability::PermissionEntry };

      tokens.append_all(match self {
        Self::PermissionRef(id) => {
          quote! { #prefix::PermissionRef(#id) }
        }
        Self::ExtendedPermission { identifier, scope } => {
          quote! { #prefix::ExtendedPermission {
            identifier: #identifier,
            scope: #scope
          } }
        }
      });
    }
  }

  impl ToTokens for Capability {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let identifier = str_lit(&self.identifier);
      let description = str_lit(&self.description);
      let context = &self.context;
      let windows = vec_lit(&self.windows, str_lit);
      let permissions = vec_lit(&self.permissions, identity);
      let platforms = vec_lit(&self.platforms, identity);

      literal_struct!(
        tokens,
        ::tauri::utils::acl::capability::Capability,
        identifier,
        description,
        context,
        windows,
        permissions,
        platforms
      );
    }
  }
}
