// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Java detection and Java/Gradle compatibility checks for Android builds.

use std::path::{Path, PathBuf};

pub fn check_java_gradle_versions() -> Option<()> {
  let java_major_version = java_major_version()?;
  let project_gradle_version = project_gradle_version()?;
  let gradle_max_supported_java = gradle_max_supported_java(&project_gradle_version)?;
  // Read the Gradle version this project actually uses (the user may have upgraded it away from
  // the template default) and only warn when the detected Java is too new for that Gradle.
  if java_major_version > gradle_max_supported_java {
    log::warn!(
          "Detected Java {java_major_version}, but Gradle {project_gradle_version} used by this project can only run on Java up to {gradle_max_supported_java}. \
           Android builds will likely fail with a cryptic error. Install a JDK that Gradle {project_gradle_version} supports \
           (Java {gradle_max_supported_java} or older) and point JAVA_HOME at it. \
           See https://docs.gradle.org/current/userguide/compatibility.html"
        );
  }
  Some(())
}

/// Reads the Gradle version this project is pinned to from its
/// `gradle/wrapper/gradle-wrapper.properties` `distributionUrl` (e.g.
/// `gradle-8.14-bin.zip` -> `8.14`). Returns `None` when the project isn't
/// generated yet or the file can't be parsed, in which case no warning is shown.
fn project_gradle_version() -> Option<String> {
  let project_path = std::env::var_os("TAURI_ANDROID_PROJECT_PATH")?;
  let properties = std::fs::read_to_string(
    Path::new(&project_path).join("gradle/wrapper/gradle-wrapper.properties"),
  )
  .ok()?;
  properties.lines().find_map(|line| {
    let url = line.trim().strip_prefix("distributionUrl=")?;
    let file = url.rsplit('/').next()?;
    let version = file.strip_prefix("gradle-")?.split('-').next()?;
    (!version.is_empty()).then(|| version.to_string())
  })
}

/// Highest Java feature version a given Gradle release can *run* on.
/// Known Gradle 7.x through 9.x releases are mapped. Unknown future major
/// versions return `None` to avoid false warnings. See
/// <https://docs.gradle.org/current/userguide/compatibility.html>.
fn gradle_max_supported_java(version: &str) -> Option<u32> {
  let mut parts = version.split('.');
  let major: u32 = parts.next()?.parse().ok()?;
  let minor: u32 = parts.next().and_then(|m| m.parse().ok()).unwrap_or(0);

  if major >= 10 {
    return None;
  }

  Some(match (major, minor) {
    v if v >= (9, 4) => 26,
    v if v >= (9, 1) => 25,
    v if v >= (8, 14) => 24,
    v if v >= (8, 10) => 23,
    v if v >= (8, 8) => 22,
    v if v >= (8, 5) => 21,
    v if v >= (8, 3) => 20,
    v if v >= (7, 6) => 19,
    v if v >= (7, 5) => 18,
    v if v >= (7, 3) => 17,
    _ => return None,
  })
}

/// Detects the major (feature) version of the active Java installation by
/// reading the `release` metadata file shipped inside every JDK, preferring
/// `JAVA_HOME` and falling back to the `java` binary on `PATH`. This avoids
/// spawning a `java -version` subprocess on every Android command.
fn java_major_version() -> Option<u32> {
  let release = std::fs::read_to_string(java_home_dir()?.join("release")).ok()?;
  release
    .lines()
    .find_map(|line| line.strip_prefix("JAVA_VERSION="))
    .and_then(parse_java_major)
}

/// Resolves the active JDK home directory, preferring `JAVA_HOME` and otherwise
/// deriving it from the `java` binary on `PATH` (`<home>/bin/java` -> `<home>`).
fn java_home_dir() -> Option<PathBuf> {
  if let Some(home) = std::env::var_os("JAVA_HOME") {
    return Some(PathBuf::from(home));
  }
  let java = which::which("java").ok()?;
  java.parent()?.parent().map(Path::to_path_buf)
}

/// Parses the major (feature) version out of a quoted Java version string (the
/// `JAVA_VERSION="..."` line of a JDK `release` file, or a `java -version`
/// banner), handling both the legacy `1.x` scheme (`1.8.0_292` -> 8) and the
/// modern scheme (`21.0.1` -> 21).
fn parse_java_major(version_output: &str) -> Option<u32> {
  let start = version_output.find('"')?;
  let rest = &version_output[start + 1..];
  let end = rest.find('"')?;
  let mut parts = rest[..end].split(['.', '_', '-']);

  let first = parts.next()?;
  if first == "1" {
    parts.next()?.parse().ok()
  } else {
    first.parse().ok()
  }
}

#[cfg(test)]
mod tests {
  use super::{gradle_max_supported_java, parse_java_major};

  #[test]
  fn parses_java_major_version() {
    // modern scheme
    assert_eq!(
      parse_java_major("openjdk version \"21.0.1\" 2023-10-17"),
      Some(21)
    );
    assert_eq!(
      parse_java_major("openjdk version \"26\" 2026-03-17"),
      Some(26)
    );
    assert_eq!(
      parse_java_major("java version \"17.0.9\" 2023-10-17 LTS"),
      Some(17)
    );
    // legacy 1.x scheme
    assert_eq!(parse_java_major("java version \"1.8.0_292\""), Some(8));
    // JDK `release` file line (the path java_major_version actually feeds in)
    assert_eq!(parse_java_major("\"21.0.1\""), Some(21));
    // garbage
    assert_eq!(parse_java_major("no version here"), None);
  }

  #[test]
  fn maps_gradle_to_max_java() {
    assert_eq!(gradle_max_supported_java("9.6.1"), Some(26));
    assert_eq!(gradle_max_supported_java("9.4.0"), Some(26));
    assert_eq!(gradle_max_supported_java("9.3"), Some(25));
    assert_eq!(gradle_max_supported_java("9.1.0"), Some(25));
    assert_eq!(gradle_max_supported_java("9.0"), Some(24));
    assert_eq!(gradle_max_supported_java("8.14"), Some(24));
    assert_eq!(gradle_max_supported_java("8.5"), Some(21));
    assert_eq!(gradle_max_supported_java("7.3"), Some(17));
    // pre-7.3 is unmapped
    assert_eq!(gradle_max_supported_java("7.2"), None);
    // Unknown future Gradle majors are left unmapped to avoid false warnings.
    assert_eq!(gradle_max_supported_java("10.0"), None);
    // unparsable
    assert_eq!(gradle_max_supported_java("not-a-version"), None);
  }
}
