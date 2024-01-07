use cargo_metadata::{Metadata, MetadataCommand};
use thiserror::Error;

pub mod acl;

#[derive(Debug, Error)]
pub enum Error {
  #[error("expected build script env var {0}, but it was not found - ensure this is called in a build script")]
  BuildVar(String),

  #[error("plugin names cannot contain underscores")]
  CrateName,

  #[error("package.links field in the Cargo manifest is not set, it should be set to the same as package.name")]
  LinksMissing,

  #[error(
    "package.links field in the Cargo manifest MUST be set to the same value as package.name"
  )]
  LinksName,

  #[error("CARGO_MANIFEST_DIR could not canonicalize")]
  Manifest(std::io::Error),

  #[error("failed to read file: {0}")]
  ReadFile(std::io::Error),

  #[error("failed to write file: {0}")]
  WriteFile(std::io::Error),

  #[error("failed to execute: {0}")]
  Metadata(#[from] cargo_metadata::Error),

  #[error("failed to run glob: {0}")]
  Glob(#[from] glob::PatternError),

  #[error("failed to parse TOML: {0}")]
  Toml(#[from] toml::de::Error),

  #[error("failed to parse JSON: {0}")]
  Json(#[from] serde_json::Error),

  #[error("unknown permission format {0}")]
  UnknownPermissionFormat(String),
}

/// [`try_build`] but will exit automatically if an error is found.
pub fn build(permissions_path_pattern: &str) {
  if let Err(error) = try_build(permissions_path_pattern) {
    println!("{}: {}", env!("CARGO_PKG_NAME"), error);
    std::process::exit(1);
  }
}

/// Ensure this crate is properly configured to be a Tauri plugin.
///
/// # Errors
///
/// Errors will occur if environmental variables expected to be set inside of [build scripts]
/// are not found, or if the crate violates Tauri plugin conventions.
pub fn try_build(permissions_path_pattern: &str) -> Result<(), Error> {
  // convention: plugin names should not use underscores
  let name = build_var("CARGO_PKG_NAME")?;
  if name.contains('_') {
    return Err(Error::CrateName);
  }

  // requirement: links MUST be set and MUST match the name
  let _links = build_var("CARGO_MANIFEST_LINKS")?;

  acl::define_permissions(permissions_path_pattern)?;

  let metadata = find_metadata()?;
  println!("{metadata:#?}");

  Ok(())
}

/// Grab an env var that is expected to be set inside of build scripts.
fn build_var(key: &str) -> Result<String, Error> {
  std::env::var(key).map_err(|_| Error::BuildVar(key.into()))
}

fn find_metadata() -> Result<Metadata, Error> {
  build_var("CARGO_MANIFEST_DIR")
    .and_then(|p| std::fs::canonicalize(p).map_err(Error::Manifest))
    .and_then(|dir| {
      MetadataCommand::new()
        .current_dir(dir)
        .no_deps()
        .exec()
        .map_err(Error::Metadata)
    })
}
