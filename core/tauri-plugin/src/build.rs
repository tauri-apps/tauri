use anyhow::{bail, ensure, Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};

/// [`try_build`] but will exit automatically if an error is found.
pub fn build() {
  if let Err(error) = try_build() {
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
pub fn try_build() -> Result<()> {
  // convention: plugin names should not use underscores
  let name = build_var("CARGO_PKG_NAME")?;
  ensure!(
    !name.contains('_'),
    "names of Tauri plugins should not use underscores, only hyphens"
  );

  // requirement: links MUST be set and MUST match the name
  match build_var("CARGO_MANIFEST_LINKS") {
    Ok(links) => ensure!(name == links, "package.links field in the Cargo manifest MUST be set to the same value as package.name"),
    Err(_) => bail!("package.links field in the Cargo manifest is not set, it should be set to the same as package.name")
  }

  let metadata = find_metadata()?;
  println!("{metadata:#?}");

  Ok(())
}

/// Grab an env var that is expected to be set inside of build scripts.
fn build_var(key: &str) -> Result<String> {
  std::env::var(key).with_context(|| format!("expected build script env var {key}, but it was not found - ensure this is called in a build script"))
}

fn find_metadata() -> Result<Metadata> {
  build_var("CARGO_MANIFEST_DIR")
    .and_then(|p| {
      std::fs::canonicalize(p).context("CARGO_MANIFEST_DIR was not able to be canonicalized")
    })
    .and_then(|dir| {
      MetadataCommand::new()
        .current_dir(dir)
        .no_deps()
        .exec()
        .context("failed to grab Cargo manifest")
    })
}
