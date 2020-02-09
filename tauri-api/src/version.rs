use semver::Version;
use std::cmp::Ordering;

/// Compare two semver versions
pub fn compare(first: &str, second: &str) -> crate::Result<i32> {
  let v1 = Version::parse(first)?;
  let v2 = Version::parse(second)?;
  match v1.cmp(&v2) {
    Ordering::Greater => Ok(-1),
    Ordering::Less => Ok(1),
    Ordering::Equal => Ok(0)
  }
}

/// Check if the "second" semver is compatible with the "first"
pub fn is_compatible(first: &str, second: &str) -> crate::Result<bool> {
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

/// Check if a the "other" version is a major bump from the "current"
pub fn is_major(current: &str, other: &str) -> crate::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(other.major > current.major)
}

/// Check if a the "other" version is a minor bump from the "current"
pub fn is_minor(current: &str, other: &str) -> crate::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(current.major == other.major && other.minor > current.minor)
}

/// Check if a the "other" version is a patch bump from the "current"
pub fn is_patch(current: &str, other: &str) -> crate::Result<bool> {
  let current = Version::parse(current)?;
  let other = Version::parse(other)?;
  Ok(current.major == other.major && current.minor == other.minor && other.patch > current.patch)
}
