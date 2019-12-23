use std::env;
use std::fs;
use std::path::PathBuf;

use crate::http;
use tauri_api::file::{Extract, Move};

pub mod github;

mod error;
pub use error::Error;

/// Status returned after updating
///
/// Wrapped `String`s are version tags
#[derive(Debug, Clone)]
pub enum Status {
  UpToDate(String),
  Updated(String),
}
impl Status {
  /// Return the version tag
  pub fn version(&self) -> &str {
    use Status::*;
    match *self {
      UpToDate(ref s) => s,
      Updated(ref s) => s,
    }
  }

  /// Returns `true` if `Status::UpToDate`
  pub fn uptodate(&self) -> bool {
    match *self {
      Status::UpToDate(_) => true,
      _ => false,
    }
  }

  /// Returns `true` if `Status::Updated`
  pub fn updated(&self) -> bool {
    match *self {
      Status::Updated(_) => true,
      _ => false,
    }
  }
}

#[derive(Clone, Debug)]
pub struct Release {
  pub version: String,
  pub asset_name: String,
  pub download_url: String,
}

#[derive(Debug)]
pub struct UpdateBuilder {
  release: Option<Release>,
  bin_name: Option<String>,
  bin_install_path: Option<PathBuf>,
  bin_path_in_archive: Option<PathBuf>,
  show_download_progress: bool,
  show_output: bool,
  current_version: Option<String>,
}
impl UpdateBuilder {
  /// Initialize a new builder, defaulting the `bin_install_path` to the current
  /// executable's path
  ///
  /// * Errors:
  ///     * Io - Determining current exe path
  pub fn new() -> Result<Self, Error> {
    Ok(Self {
      release: None,
      bin_name: None,
      bin_install_path: Some(env::current_exe()?),
      bin_path_in_archive: None,
      show_download_progress: false,
      show_output: true,
      current_version: None,
    })
  }

  pub fn release(&mut self, release: Release) -> &mut Self {
    self.release = Some(release);
    self
  }

  /// Set the current app version, used to compare against the latest available version.
  /// The `cargo_crate_version!` macro can be used to pull the version from your `Cargo.toml`
  pub fn current_version(&mut self, ver: &str) -> &mut Self {
    self.current_version = Some(ver.to_owned());
    self
  }

  /// Set the exe's name. Also sets `bin_path_in_archive` if it hasn't already been set.
  pub fn bin_name(&mut self, name: &str) -> &mut Self {
    self.bin_name = Some(name.to_owned());
    if self.bin_path_in_archive.is_none() {
      self.bin_path_in_archive = Some(PathBuf::from(name));
    }
    self
  }

  /// Set the installation path for the new exe, defaults to the current
  /// executable's path
  pub fn bin_install_path(&mut self, bin_install_path: &str) -> &mut Self {
    self.bin_install_path = Some(PathBuf::from(bin_install_path));
    self
  }

  /// Set the path of the exe inside the release tarball. This is the location
  /// of the executable relative to the base of the tar'd directory and is the
  /// path that will be copied to the `bin_install_path`. If not specified, this
  /// will default to the value of `bin_name`. This only needs to be specified if
  /// the path to the binary (from the root of the tarball) is not equal to just
  /// the `bin_name`.
  ///
  /// # Example
  ///
  /// For a tarball `myapp.tar.gz` with the contents:
  ///
  /// ```shell
  /// myapp.tar/
  ///  |------- bin/
  ///  |         |--- myapp  # <-- executable
  /// ```
  ///
  /// The path provided should be:
  ///
  /// ```
  /// # use tauri::updater::Update;
  /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
  /// Update::configure()?
  ///     .bin_path_in_archive("bin/myapp")
  /// #   .build()?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn bin_path_in_archive(&mut self, bin_path: &str) -> &mut Self {
    self.bin_path_in_archive = Some(PathBuf::from(bin_path));
    self
  }

  /// Toggle download progress bar, defaults to `off`.
  pub fn show_download_progress(&mut self, show: bool) -> &mut Self {
    self.show_download_progress = show;
    self
  }

  /// Toggle update output information, defaults to `true`.
  pub fn show_output(&mut self, show: bool) -> &mut Self {
    self.show_output = show;
    self
  }

  /// Confirm config and create a ready-to-use `Update`
  ///
  /// * Errors:
  ///     * Config - Invalid `Update` configuration
  pub fn build(&self) -> Result<Update, Error> {
    Ok(Update {
      release: if let Some(ref release) = self.release {
        release.to_owned()
      } else {
        bail!(Error::Config, "`release` required")
      },
      bin_name: if let Some(ref name) = self.bin_name {
        name.to_owned()
      } else {
        bail!(Error::Config, "`bin_name` required")
      },
      bin_install_path: if let Some(ref path) = self.bin_install_path {
        path.to_owned()
      } else {
        bail!(Error::Config, "`bin_install_path` required")
      },
      bin_path_in_archive: if let Some(ref path) = self.bin_path_in_archive {
        path.to_owned()
      } else {
        bail!(Error::Config, "`bin_path_in_archive` required")
      },
      current_version: if let Some(ref ver) = self.current_version {
        ver.to_owned()
      } else {
        bail!(Error::Config, "`current_version` required")
      },
      show_download_progress: self.show_download_progress,
      show_output: self.show_output,
    })
  }
}

/// Updates to a specified or latest release distributed
#[derive(Debug)]
pub struct Update {
  release: Release,
  current_version: String,
  bin_name: String,
  bin_install_path: PathBuf,
  bin_path_in_archive: PathBuf,
  show_download_progress: bool,
  show_output: bool,
}
impl Update {
  /// Initialize a new `Update` builder
  pub fn configure() -> Result<UpdateBuilder, Error> {
    UpdateBuilder::new()
  }

  fn print_flush(&self, msg: &str) -> Result<(), Error> {
    if self.show_output {
      print_flush!("{}", msg);
    }
    Ok(())
  }

  fn println(&self, msg: &str) {
    if self.show_output {
      println!("{}", msg);
    }
  }

  pub fn update(self) -> Result<Status, Error> {
    self.println(&format!(
      "Checking current version... v{}",
      self.current_version
    ));

    if self.show_output {
      println!("\n{} release status:", self.bin_name);
      println!("  * Current exe: {:?}", self.bin_install_path);
      println!("  * New exe download url: {:?}", self.release.download_url);
      println!(
        "\nThe new release will be downloaded/extracted and the existing binary will be replaced."
      );
    }

    let tmp_dir_parent = self
      .bin_install_path
      .parent()
      .ok_or_else(|| Error::Updater("Failed to determine parent dir".into()))?;
    let tmp_dir =
      tempdir::TempDir::new_in(&tmp_dir_parent, &format!("{}_download", self.bin_name))?;
    let tmp_archive_path = tmp_dir.path().join(&self.release.asset_name);
    let mut tmp_archive = fs::File::create(&tmp_archive_path)?;

    self.println("Downloading...");
    http::download(
      &self.release.download_url,
      &mut tmp_archive,
      self.show_download_progress,
    )?;

    self.print_flush("Extracting archive... ")?;
    Extract::from_source(&tmp_archive_path)
      .extract_file(&tmp_dir.path(), &self.bin_path_in_archive)?;
    let new_exe = tmp_dir.path().join(&self.bin_path_in_archive);
    self.println("Done");

    self.print_flush("Replacing binary file... ")?;
    let tmp_file = tmp_dir.path().join(&format!("__{}_backup", self.bin_name));
    Move::from_source(&new_exe)
      .replace_using_temp(&tmp_file)
      .to_dest(&self.bin_install_path)?;
    self.println("Done");
    Ok(Status::Updated(self.release.version))
  }
}
