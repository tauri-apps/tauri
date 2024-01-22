// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access Control List types.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use thiserror::Error;

pub use self::{identifier::*, value::*};

#[cfg(feature = "build")]
pub mod build;
pub mod capability;
pub mod identifier;
pub mod plugin;
pub mod resolved;
pub mod value;

/// Possible errors while processing ACL files.
#[derive(Debug, Error)]
pub enum Error {
  /// Could not find an environmental variable that is set inside of build scripts.
  ///
  /// Whatever generated this should be called inside of a build script.
  #[error("expected build script env var {0}, but it was not found - ensure this is called in a build script")]
  BuildVar(&'static str),

  /// Plugin name doesn't follow Tauri standards
  #[error("plugin names cannot contain underscores")]
  CrateName,

  /// The links field in the manifest **MUST** be set and match the name of the crate.
  #[error("package.links field in the Cargo manifest is not set, it should be set to the same as package.name")]
  LinksMissing,

  /// The links field in the manifest **MUST** match the name of the crate.
  #[error(
    "package.links field in the Cargo manifest MUST be set to the same value as package.name"
  )]
  LinksName,

  /// IO error while reading a file
  #[error("failed to read file: {0}")]
  ReadFile(std::io::Error),

  /// IO error while writing a file
  #[error("failed to write file: {0}")]
  WriteFile(std::io::Error),

  /// IO error while creating a file
  #[error("failed to create file: {0}")]
  CreateFile(std::io::Error),

  /// [`cargo_metadata`] was not able to complete successfully
  #[cfg(feature = "build")]
  #[error("failed to execute: {0}")]
  Metadata(#[from] ::cargo_metadata::Error),

  /// Invalid glob
  #[error("failed to run glob: {0}")]
  Glob(#[from] glob::PatternError),

  /// Invalid TOML encountered
  #[error("failed to parse TOML: {0}")]
  Toml(#[from] toml::de::Error),

  /// Invalid JSON encountered
  #[error("failed to parse JSON: {0}")]
  Json(#[from] serde_json::Error),

  /// Invalid permissions file format
  #[error("unknown permission format {0}")]
  UnknownPermissionFormat(String),

  /// Invalid capabilities file format
  #[error("unknown capability format {0}")]
  UnknownCapabilityFormat(String),

  /// Permission referenced in set not found.
  #[error("permission {permission} not found from set {set}")]
  SetPermissionNotFound {
    /// Permission identifier.
    permission: String,
    /// Set identifier.
    set: String,
  },

  /// Plugin has no default permission.
  #[error("plugin {plugin} has no default permission")]
  MissingDefaultPermission {
    /// Plugin name.
    plugin: String,
  },

  /// Unknown plugin.
  #[error("unknown plugin {plugin}, expected one of {available}")]
  UnknownPlugin {
    /// Plugin name.
    plugin: String,
    /// Available plugins.
    available: String,
  },

  /// Unknown permission.
  #[error("unknown permission {permission} for plugin {plugin}")]
  UnknownPermission {
    /// Plugin name.
    plugin: String,

    /// Permission identifier.
    permission: String,
  },
}

/// Allowed and denied commands inside a permission.
///
/// If two commands clash inside of `allow` and `deny`, it should be denied by default.
#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Commands {
  /// Allowed command.
  #[serde(default)]
  pub allow: Vec<String>,

  /// Denied command, which takes priority.
  #[serde(default)]
  pub deny: Vec<String>,
}

/// A restriction of the command/endpoint functionality.
///
/// It can be of any serde serializable type and is used for allowing or preventing certain actions inside a Tauri command.
///
/// The scope is passed to the command and handled/enforced by the command itself.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Scopes {
  /// Data that defines what is allowed by the scope.
  pub allow: Option<Vec<Value>>,
  /// Data that defines what is denied by the scope.
  pub deny: Option<Vec<Value>>,
}

/// Descriptions of explicit privileges of commands.
///
/// It can enable commands to be accessible in the frontend of the application.
///
/// If the scope is defined it can be used to fine grain control the access of individual or multiple commands.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Permission {
  /// The version of the permission.
  pub version: Option<NonZeroU64>,

  /// A unique identifier for the permission.
  pub identifier: String,

  /// Human-readable description of what the permission does.
  pub description: Option<String>,

  /// Allowed or denied commands when using this permission.
  #[serde(default)]
  pub commands: Commands,

  /// Allowed or denied scoped when using this permission.
  #[serde(default)]
  pub scope: Scopes,
}

/// A set of direct permissions grouped together under a new name.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  pub identifier: String,

  /// Human-readable description of what the permission does.
  pub description: String,

  /// All permissions this set contains.
  pub permissions: Vec<String>,
}

/// Execution context of an IPC call.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ExecutionContext {
  /// A local URL is used (the Tauri app URL).
  Local,
  /// Remote URL is tring to use the IPC.
  Remote {
    /// The domain trying to access the IPC.
    domain: String,
  },
}

#[cfg(feature = "build")]
mod build_ {
  use crate::tokens::*;

  use super::*;
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  impl ToTokens for ExecutionContext {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::ExecutionContext };

      tokens.append_all(match self {
        Self::Local => {
          quote! { #prefix::Local }
        }
        Self::Remote { domain } => {
          let domain = str_lit(domain);
          quote! { #prefix::Remote { domain: #domain } }
        }
      });
    }
  }
}
