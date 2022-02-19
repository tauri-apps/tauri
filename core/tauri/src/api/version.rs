// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Compare two semantic versions.
//!
//! [Semantic Versioning](https://semver.org) is a guideline for how version numbers are assigned and incremented.
//! The functions on this module are helpers around [semver](https://docs.rs/semver/latest/semver/).

use semver::Version;
use std::cmp::Ordering;

/// Compare two semver versions.
///
/// If the `first` semver is greater, returns -1.
/// If the `second` semver is greater, returns 1.
/// If they are equal, returns 0.
///
/// # Examples
///
/// ```
/// use tauri::api::version::compare;
/// assert_eq!(compare("0.15.0", "0.15.5").unwrap(), 1);
/// assert_eq!(compare("0.15.10", "0.15.9").unwrap(), -1);
/// assert_eq!(compare("0.15.10", "0.16.10").unwrap(), 1);
/// assert_eq!(compare("1.57.0", "2.17.4").unwrap(), 1);
/// assert_eq!(compare("0.0.0", "0.0.0").unwrap(), 0);
/// ```
pub fn compare(first: &str, second: &str) -> crate::api::Result<i8> {
  let v1 = Version::parse(first)?;
  let v2 = Version::parse(second)?;
  match v1.cmp(&v2) {
    Ordering::Greater => Ok(-1),
    Ordering::Less => Ok(1),
    Ordering::Equal => Ok(0),
  }
}

/// Check if the "second" semver is compatible with the "first".
///
/// # Examples
///
/// ```
/// use tauri::api::version::is_compatible;
/// assert!(is_compatible("0.15.0", "0.15.5").unwrap());
/// assert!(!is_compatible("0.15.0", "0.16.5").unwrap());
///
/// assert!(is_compatible("1.5.0", "1.5.10").unwrap());
/// assert!(is_compatible("1.54.0", "1.55.0").unwrap());
/// assert!(!is_compatible("2.17.0", "3.17.0").unwrap());
/// ```
pub fn is_compatible(first: &str, second: &str) -> crate::api::Result<bool> {
  let first = Version::parse(first)?;
  let second = Version::parse(second)?;
  Ok(if second.major == 0 && first.major == 0 {
    first.minor == second.minor && second.patch > first.patch
  } else if second.major > 0 {
    first.major == second.major
      && ((second.minor > first.minor)
        || (first.minor == second.minor && second.patch > first.patch))
  } else {
    false
  })
}

/// Check if a the "other" version is a major bump from the "current".
///
/// # Examples
///
/// ```
/// use tauri::api::version::is_major;
/// assert!(is_major("1.0.0", "2.0.0").unwrap());
/// assert!(is_major("1.5.0", "2.17.10").unwrap());
/// assert!(is_major("0.5.0", "2.17.10").unwrap());
/// assert!(!is_major("1.1.5", "1.2.5").unwrap());
/// assert!(!is_major("0.14.0", "0.15.0").unwrap());
/// ```
pub fn is_major(current: &str, other: &str) -> crate::api::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(other.major > current.major)
}

/// Check if a the "other" version is a minor bump from the "current".
///
/// # Examples
///
/// ```
/// use tauri::api::version::is_minor;
/// assert!(is_minor("0.15.10", "0.16.110").unwrap());
/// assert!(is_minor("1.0.0", "1.1.1").unwrap());
/// assert!(!is_minor("2.1.9", "3.2.0").unwrap());
/// assert!(!is_minor("1.0.0", "1.0.10").unwrap());
/// ```
pub fn is_minor(current: &str, other: &str) -> crate::api::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(current.major == other.major && other.minor > current.minor)
}

/// Check if a the "other" version is a patch bump from the "current".
///
/// # Examples
///
/// ```
/// use tauri::api::version::is_patch;
/// assert!(is_patch("0.15.0", "0.15.1").unwrap());
/// assert!(is_patch("1.0.0", "1.0.1").unwrap());
/// assert!(!is_patch("2.2.0", "2.3.1").unwrap());
/// assert!(!is_patch("2.2.1", "1.1.0").unwrap());
/// ```
pub fn is_patch(current: &str, other: &str) -> crate::api::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(current.major == other.major && current.minor == other.minor && other.patch > current.patch)
}

/// Check if a version is greater than the current.
///
/// # Examples
///
/// ```
/// use tauri::api::version::is_greater;
/// assert!(is_greater("0.15.10", "0.16.0").unwrap());
/// assert!(is_greater("1.0.0", "1.0.1").unwrap());
/// assert!(is_greater("1.1.9", "1.2.0").unwrap());
/// assert!(is_greater("1.0.0", "2.0.0").unwrap());
/// ```
pub fn is_greater(current: &str, other: &str) -> crate::api::Result<bool> {
  Ok(Version::parse(other)? > Version::parse(current)?)
}
