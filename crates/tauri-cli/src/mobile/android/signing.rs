// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Release signing configuration of the generated Android project.
//!
//! The app's `build.gradle.kts` reads the upload key from `keystore.properties`,
//! next to the root `build.gradle.kts`, when that file exists.

use std::{
  path::{Path, PathBuf},
  sync::OnceLock,
};

use super::{find_matching_brace, line_indentation};
use crate::{
  Result,
  error::{Context, ErrorExt},
};

pub const KEYSTORE_PROPERTIES: &str = "keystore.properties";

/// Signing config added to existing projects, matching the one in the project template.
const SIGNING_CONFIG: &str = r#"signingConfigs {
    val keystorePropertiesFile = rootProject.file("keystore.properties")
    if (keystorePropertiesFile.exists()) {
        val keystoreProperties = java.util.Properties().apply {
            keystorePropertiesFile.inputStream().use { load(it) }
        }
        create("release") {
            storeFile = rootProject.file(keystoreProperties.getProperty("storeFile"))
            storePassword = keystoreProperties.getProperty("password")
            keyAlias = keystoreProperties.getProperty("keyAlias")
            keyPassword = keystoreProperties.getProperty("keyPassword", keystoreProperties.getProperty("password"))
        }
    }
}"#;

const RELEASE_SIGNING_CONFIG: &str = r#"signingConfig = signingConfigs.findByName("release")"#;

pub struct KeystoreProperties {
  pub store_file: PathBuf,
  pub password: String,
  pub key_alias: String,
  /// Defaults to the keystore password, which is the only option for PKCS12 keystores.
  pub key_password: Option<String>,
}

impl KeystoreProperties {
  fn serialize(&self) -> String {
    let mut out = String::from(
      "# Upload key used to sign release builds. Keep this file private, do not commit it.\n",
    );
    out.push_str(&format!(
      "storeFile={}\n",
      escape_property(&self.store_file.to_string_lossy())
    ));
    out.push_str(&format!("keyAlias={}\n", escape_property(&self.key_alias)));
    out.push_str(&format!("password={}\n", escape_property(&self.password)));
    if let Some(key_password) = &self.key_password {
      out.push_str(&format!("keyPassword={}\n", escape_property(key_password)));
    }
    out
  }

  pub fn write(&self, project_dir: &Path) -> Result<PathBuf> {
    let path = project_dir.join(KEYSTORE_PROPERTIES);
    write_private(&path, self.serialize())?;
    Ok(path)
  }
}

/// Escapes a value for a Java `.properties` file.
fn escape_property(value: &str) -> String {
  let mut out = String::with_capacity(value.len());
  for (i, c) in value.chars().enumerate() {
    match c {
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      // leading whitespace is otherwise trimmed
      ' ' if i == 0 => out.push_str("\\ "),
      c => out.push(c),
    }
  }
  out
}

fn write_private(path: &Path, contents: String) -> Result<()> {
  #[cfg(unix)]
  {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(path)
      .fs_context("failed to open file", path.to_path_buf())?;
    file
      .write_all(contents.as_bytes())
      .fs_context("failed to write file", path.to_path_buf())
  }
  #[cfg(not(unix))]
  std::fs::write(path, contents).fs_context("failed to write file", path.to_path_buf())
}

#[derive(Debug, PartialEq, Eq)]
pub enum GradleSigning {
  /// `build.gradle.kts` already reads `keystore.properties`.
  Configured,
  /// The release signing config was added to `build.gradle.kts`.
  Added,
  /// `build.gradle.kts` defines its own release signing config, which is left untouched.
  Custom,
}

pub fn app_build_gradle_path(project_dir: &Path) -> PathBuf {
  project_dir.join("app").join("build.gradle.kts")
}

/// Makes sure the app's `build.gradle.kts` signs release builds with the key from `keystore.properties`.
pub fn ensure_release_signing_config(project_dir: &Path) -> Result<GradleSigning> {
  let path = app_build_gradle_path(project_dir);
  let build_gradle =
    std::fs::read_to_string(&path).fs_context("failed to read build.gradle.kts", path.clone())?;

  let (status, updated) = add_release_signing_config(&build_gradle)?;
  if let Some(updated) = updated {
    std::fs::write(&path, updated).fs_context("failed to write build.gradle.kts", path)?;
  }
  Ok(status)
}

fn release_signing_config_re() -> &'static regex::Regex {
  static RE: OnceLock<regex::Regex> = OnceLock::new();
  RE.get_or_init(|| {
    regex::Regex::new(
      r#"\bsigningConfigs\b[\s\S]*\b(create|register|maybeCreate)\(\s*"release"\s*\)"#,
    )
    .unwrap()
  })
}

fn add_release_signing_config(build_gradle: &str) -> Result<(GradleSigning, Option<String>)> {
  if build_gradle.contains(KEYSTORE_PROPERTIES) {
    return Ok((GradleSigning::Configured, None));
  }
  if release_signing_config_re().is_match(build_gradle) {
    return Ok((GradleSigning::Custom, None));
  }

  static BUILD_TYPES_RE: OnceLock<regex::Regex> = OnceLock::new();
  static RELEASE_RE: OnceLock<regex::Regex> = OnceLock::new();
  let build_types_re =
    BUILD_TYPES_RE.get_or_init(|| regex::Regex::new(r#"\bbuildTypes\s*\{"#).unwrap());
  let release_re = RELEASE_RE.get_or_init(|| {
    regex::Regex::new(r#"(?:\bgetByName\(\s*"release"\s*\)|\brelease\b)\s*\{"#).unwrap()
  });

  let build_types = build_types_re
    .find(build_gradle)
    .context("could not find the `buildTypes` block in build.gradle.kts")?;
  let build_types_open = build_types.end() - 1;
  let build_types_close = find_matching_brace(build_gradle, build_types_open)
    .context("could not find the end of the `buildTypes` block in build.gradle.kts")?;

  let indentation = line_indentation(build_gradle, build_types.start()).to_string();
  let inner_indentation = format!("{indentation}    ");
  let block_indentation = format!("{inner_indentation}    ");

  let mut updated = String::with_capacity(build_gradle.len() + SIGNING_CONFIG.len() * 2);

  // signingConfigs block right before buildTypes
  let line_start = build_gradle[..build_types.start()]
    .rfind('\n')
    .map_or(0, |i| i + 1);
  updated.push_str(&build_gradle[..line_start]);
  for line in SIGNING_CONFIG.lines() {
    updated.push_str(&indentation);
    updated.push_str(line);
    updated.push('\n');
  }

  let release = release_re
    .find(&build_gradle[build_types_open..build_types_close])
    .map(|m| build_types_open + m.end() - 1);

  match release {
    Some(release_open) => {
      updated.push_str(&build_gradle[line_start..=release_open]);
      updated.push('\n');
      updated.push_str(&block_indentation);
      updated.push_str(RELEASE_SIGNING_CONFIG);
      updated.push_str(&build_gradle[release_open + 1..]);
    }
    None => {
      updated.push_str(&build_gradle[line_start..=build_types_open]);
      updated.push('\n');
      updated.push_str(&format!(
        "{inner_indentation}getByName(\"release\") {{\n{block_indentation}{RELEASE_SIGNING_CONFIG}\n{inner_indentation}}}"
      ));
      updated.push_str(&build_gradle[build_types_open + 1..]);
    }
  }

  Ok((GradleSigning::Added, Some(updated)))
}

/// Keeps the `keystore.properties` file generated from environment variables
/// for the duration of a build.
pub struct EnvKeystore {
  properties_path: PathBuf,
  previous_properties: Option<Vec<u8>>,
  _temp_dir: Option<tempfile::TempDir>,
}

impl Drop for EnvKeystore {
  fn drop(&mut self) {
    let _ = match &self.previous_properties {
      Some(previous) => std::fs::write(&self.properties_path, previous),
      None => std::fs::remove_file(&self.properties_path),
    };
  }
}

/// Configures the release signing key from the following environment variables, useful in CI/CD platforms:
///
/// - `ANDROID_KEYSTORE`: base64 representation of the keystore, or `ANDROID_KEYSTORE_PATH` for its path
/// - `ANDROID_KEYSTORE_PASSWORD`: keystore password
/// - `ANDROID_KEY_ALIAS`: alias of the upload key, defaults to `upload`
/// - `ANDROID_KEY_PASSWORD`: password of the upload key, defaults to the keystore password
pub fn keystore_from_env(project_dir: &Path) -> Result<Option<EnvKeystore>> {
  let keystore = std::env::var_os("ANDROID_KEYSTORE").filter(|v| !v.is_empty());
  let keystore_path = std::env::var_os("ANDROID_KEYSTORE_PATH")
    .filter(|v| !v.is_empty())
    .map(PathBuf::from);
  if keystore.is_none() && keystore_path.is_none() {
    return Ok(None);
  }

  let password = std::env::var("ANDROID_KEYSTORE_PASSWORD").context(
    "ANDROID_KEYSTORE_PASSWORD must be set when signing with the ANDROID_KEYSTORE or ANDROID_KEYSTORE_PATH environment variables",
  )?;
  let key_alias = std::env::var("ANDROID_KEY_ALIAS").unwrap_or_else(|_| "upload".into());
  let key_password = std::env::var("ANDROID_KEY_PASSWORD").ok();

  let (store_file, temp_dir) = match (keystore_path, keystore) {
    (Some(path), _) => (
      dunce::canonicalize(&path).fs_context("failed to resolve ANDROID_KEYSTORE_PATH", path)?,
      None,
    ),
    (None, Some(keystore)) => {
      use base64::Engine;
      let cleaned: String = keystore
        .to_string_lossy()
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
      let decoded = base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .context("failed to decode ANDROID_KEYSTORE, it must be the base64 representation of the keystore file")?;
      let temp_dir = tempfile::tempdir().context("failed to create temporary directory")?;
      let path = temp_dir.path().join("upload.keystore");
      std::fs::write(&path, decoded).fs_context("failed to write keystore", path.clone())?;
      (path, Some(temp_dir))
    }
    (None, None) => unreachable!(),
  };

  match ensure_release_signing_config(project_dir) {
    Ok(GradleSigning::Custom) => log::warn!(
      "Your app's build.gradle.kts defines its own release signing config that does not read {KEYSTORE_PROPERTIES}, the ANDROID_KEYSTORE environment variables might be ignored"
    ),
    Ok(GradleSigning::Added) => log::info!(
      "Added the release signing config to {}",
      tauri_utils::display_path(app_build_gradle_path(project_dir))
    ),
    Ok(GradleSigning::Configured) => {}
    Err(e) => log::warn!("Failed to configure release signing in build.gradle.kts: {e}"),
  }

  let properties_path = project_dir.join(KEYSTORE_PROPERTIES);
  let previous_properties = std::fs::read(&properties_path).ok();
  if previous_properties.is_some() {
    log::info!(
      "Signing with the keystore from environment variables instead of {}",
      tauri_utils::display_path(&properties_path)
    );
  }

  KeystoreProperties {
    store_file,
    password,
    key_alias,
    key_password,
  }
  .write(project_dir)?;

  Ok(Some(EnvKeystore {
    properties_path,
    previous_properties,
    _temp_dir: temp_dir,
  }))
}

/// Whether release builds of the project are signed with an upload key.
pub fn has_release_signing(project_dir: &Path) -> bool {
  if project_dir.join(KEYSTORE_PROPERTIES).exists() {
    return true;
  }
  // a custom signing config that does not use keystore.properties
  std::fs::read_to_string(app_build_gradle_path(project_dir)).is_ok_and(|build_gradle| {
    !build_gradle.contains(KEYSTORE_PROPERTIES)
      && release_signing_config_re().is_match(&build_gradle)
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const TEMPLATE_LIKE: &str = r#"android {
    compileSdk = 36
    buildTypes {
        getByName("debug") {
            isDebuggable = true
        }
        getByName("release") {
            isMinifyEnabled = true
        }
    }
}
"#;

  #[test]
  fn adds_signing_config() {
    let (status, updated) = add_release_signing_config(TEMPLATE_LIKE).unwrap();
    assert_eq!(status, GradleSigning::Added);
    let updated = updated.unwrap();
    assert!(updated.contains(
      "    signingConfigs {\n        val keystorePropertiesFile = rootProject.file(\"keystore.properties\")"
    ));
    assert!(updated.contains(
      "        getByName(\"release\") {\n            signingConfig = signingConfigs.findByName(\"release\")\n            isMinifyEnabled = true"
    ));
    assert!(updated.find("signingConfigs {").unwrap() < updated.find("buildTypes {").unwrap());

    // running again is a no-op
    let (status, updated) = add_release_signing_config(&updated).unwrap();
    assert_eq!(status, GradleSigning::Configured);
    assert!(updated.is_none());
  }

  #[test]
  fn template_matches_signing_config() {
    let template = include_str!("../../../templates/mobile/android/app/build.gradle.kts");
    for line in SIGNING_CONFIG.lines() {
      let line = line.replace("java.util.Properties()", "Properties()");
      assert!(
        template.contains(&format!("    {line}\n")),
        "template is missing `{line}`"
      );
    }
    assert!(template.contains(RELEASE_SIGNING_CONFIG));
  }

  #[test]
  fn adds_release_build_type() {
    let build_gradle = r#"android {
    buildTypes {
        getByName("debug") {
            isDebuggable = true
        }
    }
}
"#;
    let (_, updated) = add_release_signing_config(build_gradle).unwrap();
    assert!(updated.unwrap().contains(
      "    buildTypes {\n        getByName(\"release\") {\n            signingConfig = signingConfigs.findByName(\"release\")\n        }\n        getByName(\"debug\") {"
    ));
  }

  #[test]
  fn keeps_custom_signing_config() {
    let build_gradle = r#"android {
    signingConfigs {
        create("release") {
            storeFile = file("/path/to/key.jks")
        }
    }
    buildTypes {
        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
        }
    }
}
"#;
    let (status, updated) = add_release_signing_config(build_gradle).unwrap();
    assert_eq!(status, GradleSigning::Custom);
    assert!(updated.is_none());
  }

  #[test]
  fn debug_signing_config_is_not_custom_release() {
    let build_gradle = r#"android {
    signingConfigs {
        getByName("debug") {
            storeFile = file("debug.keystore")
        }
    }
    buildTypes {
        getByName("release") {
        }
    }
}
"#;
    let (status, _) = add_release_signing_config(build_gradle).unwrap();
    assert_eq!(status, GradleSigning::Added);
  }

  #[test]
  fn serializes_properties() {
    let properties = KeystoreProperties {
      store_file: PathBuf::from(r"C:\Users\me\upload.jks"),
      password: "pa ss".into(),
      key_alias: "upload".into(),
      key_password: None,
    };
    assert_eq!(
      properties.serialize().lines().skip(1).collect::<Vec<_>>(),
      vec![
        r"storeFile=C:\\Users\\me\\upload.jks",
        "keyAlias=upload",
        "password=pa ss"
      ]
    );
  }

  #[test]
  fn env_keystore_restores_previous_properties() {
    let dir = tempfile::tempdir().unwrap();
    let app_dir = dir.path().join("app");
    std::fs::create_dir_all(&app_dir).unwrap();
    std::fs::write(app_dir.join("build.gradle.kts"), TEMPLATE_LIKE).unwrap();
    std::fs::write(dir.path().join(KEYSTORE_PROPERTIES), "previous").unwrap();

    let keystore = dir.path().join("upload.jks");
    std::fs::write(&keystore, "keystore").unwrap();

    // SAFETY: the test does not run in parallel with other tests reading these variables
    unsafe {
      std::env::set_var("ANDROID_KEYSTORE_PATH", &keystore);
      std::env::set_var("ANDROID_KEYSTORE_PASSWORD", "secret");
    }
    let guard = keystore_from_env(dir.path()).unwrap().unwrap();
    unsafe {
      std::env::remove_var("ANDROID_KEYSTORE_PATH");
      std::env::remove_var("ANDROID_KEYSTORE_PASSWORD");
    }

    let properties = std::fs::read_to_string(dir.path().join(KEYSTORE_PROPERTIES)).unwrap();
    assert!(properties.contains("password=secret"));
    assert!(properties.contains("keyAlias=upload"));
    assert!(
      std::fs::read_to_string(app_dir.join("build.gradle.kts"))
        .unwrap()
        .contains(KEYSTORE_PROPERTIES)
    );

    drop(guard);
    assert_eq!(
      std::fs::read_to_string(dir.path().join(KEYSTORE_PROPERTIES)).unwrap(),
      "previous"
    );
  }
}
