use ignore::WalkBuilder;
use std::{fs, path};

/// Moves a file from the given path to the specified destination.
///
/// `source` and `dest` must be on the same filesystem.
/// If `replace_using_temp` is specified, the destination file will be
/// replaced using the given temporary path.
///
/// * Errors:
///     * Io - copying / renaming
#[derive(Debug)]
pub struct Move<'a> {
  source: &'a path::Path,
  temp: Option<&'a path::Path>,
}
impl<'a> Move<'a> {
  /// Specify source file
  pub fn from_source(source: &'a path::Path) -> Move<'a> {
    Self { source, temp: None }
  }

  /// If specified and the destination file already exists, the "destination"
  /// file will be moved to the given temporary location before the "source"
  /// file is moved to the "destination" file.
  ///
  /// In the event of an `io` error while renaming "source" to "destination",
  /// the temporary file will be moved back to "destination".
  ///
  /// The `temp` dir must be explicitly provided since `rename` operations require
  /// files to live on the same filesystem.
  pub fn replace_using_temp(&mut self, temp: &'a path::Path) -> &mut Self {
    self.temp = Some(temp);
    self
  }

  /// Move source file to specified destination (replace whole directory)
  pub fn to_dest(&self, dest: &path::Path) -> crate::Result<()> {
    match self.temp {
      None => {
        fs::rename(self.source, dest)?;
      }
      Some(temp) => {
        if dest.exists() {
          fs::rename(dest, temp)?;
          if let Err(e) = fs::rename(self.source, dest) {
            fs::rename(temp, dest)?;
            return Err(e.into());
          }
        } else {
          fs::rename(self.source, dest)?;
        }
      }
    };
    Ok(())
  }

  /// Walk in the source and copy all files and create directories if needed by
  /// replacing existing elements. (equivalent to a cp -R)
  pub fn walk_to_dest(&self, dest: &path::Path) -> crate::Result<()> {
    match self.temp {
      None => {
        // got no temp -- no need to backup
        walkdir_and_copy(self.source, dest)?;
      }
      Some(temp) => {
        if dest.exists() {
          // we got temp and our dest exist, lets make a backup
          // of current files
          walkdir_and_copy(dest, temp)?;

          if let Err(e) = walkdir_and_copy(self.source, dest) {
            // if we got something wrong we reset the dest with our backup
            fs::rename(temp, dest)?;
            return Err(e);
          }
        } else {
          // got temp but dest didnt exist
          walkdir_and_copy(self.source, dest)?;
        }
      }
    };
    Ok(())
  }
}
// Walk into the source and create directories, and copy files
// Overwriting existing items but keeping untouched the files in the dest
// not provided in the source.
fn walkdir_and_copy(source: &path::Path, dest: &path::Path) -> crate::Result<()> {
  let walkdir = WalkBuilder::new(source).hidden(false).build();

  for entry in walkdir {
    // Check if it's a file

    let element = entry?;
    let metadata = element.metadata()?;
    let destination = dest.join(element.path().strip_prefix(&source)?);

    // we make sure it's a directory and destination doesnt exist
    if metadata.is_dir() && !&destination.exists() {
      fs::create_dir_all(&destination)?;
    }

    // we make sure it's a file
    if metadata.is_file() {
      fs::copy(element.path(), destination)?;
    }
  }
  Ok(())
}
