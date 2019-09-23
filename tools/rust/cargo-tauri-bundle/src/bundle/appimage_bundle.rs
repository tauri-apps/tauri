use super::common;
use super::deb_bundle;
use crate::{ResultExt, Settings};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
    // execute deb bundler
    let package_path = deb_bundle::bundle_project(&settings);
    package_path
}
