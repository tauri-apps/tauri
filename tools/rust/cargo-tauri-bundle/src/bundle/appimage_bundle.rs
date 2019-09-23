use super::common;
use super::deb_bundle;
use crate::{ResultExt, Settings};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::io::{BufRead, BufReader, Cursor, Read};

use std::path::{Path, PathBuf};

pub const SH_URL: &str =
  "https://raw.githubusercontent.com/AppImage/pkg2appimage/master/pkg2appimage";

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // execute deb bundler
  let package_path = deb_bundle::bundle_project(&settings);
  package_path
}

fn get_write_pkg2appimage_sh(path: &Path) -> crate::Result<()> {
  common::print_info("Downloading and writing pkg2appimage.sh")?;

  let mut data: Vec<u8> = Vec::new();
  let mut response = reqwest::get(SH_URL).or_else(|e| Err(e.to_string()))?;

  response
    .read_to_end(&mut data)
    .or_else(|e| Err(e.to_string()))?;

  let mut sh_script = File::create(path.join(Path::new("pkg2appimage.sh")))?;
  sh_script.write_all(&data)?;

  Ok(())
}

#[test]
fn check_download() {
  let path = Path::new("./target/appimage/");
  fs::create_dir(&path).unwrap();
  let result = get_write_pkg2appimage_sh(&path).unwrap();
  assert_eq!(result, ());
}
