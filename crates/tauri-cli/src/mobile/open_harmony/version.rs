// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use semver::Version;

use crate::{error::Context, helpers::config::Config, ErrorExt, Result};

const MAX_VERSION_CODE: u64 = i32::MAX as u64;

pub(crate) struct AppVersion {
  pub(super) name: String,
  pub(super) code: u32,
}

impl AppVersion {
  pub(crate) fn from_config(config: &Config, package_version: &str) -> Result<Self> {
    let version = Version::parse(config.version.as_deref().unwrap_or(package_version))
      .context("invalid OpenHarmony application version")?;
    let mut name = format!("{}.{}.{}", version.major, version.minor, version.patch);
    if !version.pre.is_empty() {
      name.push('.');
      name.push_str(&version.pre.as_str().replace('-', "_"));
    }
    if !version.build.is_empty() {
      name.push('_');
      name.push_str(&version.build.as_str().replace('-', "_"));
    }
    if name.len() > 127 {
      crate::error::bail!("OpenHarmony versionName must not exceed 127 bytes");
    }

    let code = if let Some(code) = config.bundle.open_harmony.version_code {
      u64::from(code)
    } else {
      if version.minor >= 1000 || version.patch >= 1000 {
        crate::error::bail!(
          "OpenHarmony's default versionCode requires minor and patch below 1000; set bundle.openHarmony.versionCode explicitly"
        );
      }
      version
        .major
        .checked_mul(1_000_000)
        .and_then(|code| code.checked_add(version.minor * 1000 + version.patch))
        .context(
          "OpenHarmony versionCode overflow; set bundle.openHarmony.versionCode explicitly",
        )?
    };
    if code > MAX_VERSION_CODE {
      crate::error::bail!("OpenHarmony versionCode must be between 0 and {MAX_VERSION_CODE}");
    }

    Ok(Self {
      name,
      code: code as u32,
    })
  }

  pub(super) fn synchronize(&self, project_dir: &Path) -> Result<()> {
    let path = project_dir.join("AppScope/app.json5");
    let source = fs::read_to_string(&path).fs_context(
      "failed to read OpenHarmony application manifest",
      path.clone(),
    )?;
    let contents = self.update_manifest(&source)?;
    if contents != source {
      // Replace the complete result atomically and retain native file permissions.
      let mut output = tempfile::NamedTempFile::new_in(path.parent().unwrap()).fs_context(
        "failed to create temporary OpenHarmony manifest",
        path.clone(),
      )?;
      let permissions = fs::metadata(&path)
        .fs_context(
          "failed to read OpenHarmony manifest permissions",
          path.clone(),
        )?
        .permissions();
      fs::set_permissions(output.path(), permissions).fs_context(
        "failed to retain OpenHarmony manifest permissions",
        path.clone(),
      )?;
      std::io::Write::write_all(&mut output, contents.as_bytes()).fs_context(
        "failed to write temporary OpenHarmony manifest",
        path.clone(),
      )?;
      output
        .persist(&path)
        .map_err(|error| error.error)
        .fs_context("failed to write OpenHarmony application manifest", path)?;
    }
    Ok(())
  }

  fn update_manifest(&self, source: &str) -> Result<String> {
    // JSON5 object literals are also JavaScript expressions. Validate JSON5
    // first, then use the existing JS parser's byte spans to change only the
    // two values. Native fields, comments, quoting and whitespace stay intact.
    json5::from_str::<serde::de::IgnoredAny>(source)
      .context("failed to parse OpenHarmony application manifest as JSON5")?;
    let wrapped = format!("({source}\n)");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &wrapped, SourceType::default()).parse();
    if !parsed.errors.is_empty() {
      crate::error::bail!("failed to locate OpenHarmony application manifest fields");
    }
    let Some(Statement::ExpressionStatement(statement)) = parsed.program.body.first() else {
      crate::error::bail!("OpenHarmony application manifest must be an object");
    };
    let Expression::ObjectExpression(root) = statement.expression.get_inner_expression() else {
      crate::error::bail!("OpenHarmony application manifest must be an object");
    };
    let Expression::ObjectExpression(app) = field(root, "app")? else {
      crate::error::bail!("OpenHarmony application manifest app must be an object");
    };

    let name =
      serde_json::to_string(&self.name).context("failed to encode OpenHarmony versionName")?;
    let mut replacements = Vec::new();
    for (key, replacement) in [
      ("versionName", name),
      ("versionCode", self.code.to_string()),
    ] {
      let span = field(app, key)?.span();
      // Account for the opening parenthesis added above. Oxc spans are bytes,
      // so non-ASCII application labels do not shift the edits.
      let range = (span.start as usize - 1)..(span.end as usize - 1);
      let value = json5::from_str::<serde_json::Value>(&source[range.clone()])
        .context("invalid OpenHarmony version field")?;
      if value.is_object() || value.is_array() {
        crate::error::bail!("OpenHarmony {key} must be a scalar value");
      }
      let new_value = serde_json::from_str::<serde_json::Value>(&replacement)
        .context("failed to encode OpenHarmony version field")?;
      if value != new_value {
        replacements.push((range, replacement));
      }
    }
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut contents = source.to_owned();
    for (range, replacement) in replacements {
      contents.replace_range(range, &replacement);
    }
    Ok(contents)
  }
}

fn field<'a, 'b>(object: &'a ObjectExpression<'b>, name: &str) -> Result<&'a Expression<'b>> {
  let mut matches = object
    .properties
    .iter()
    .filter_map(|property| match property {
      ObjectPropertyKind::ObjectProperty(property)
        if property.key.is_specific_static_name(name) =>
      {
        Some(&property.value)
      }
      _ => None,
    });
  let value = matches
    .next()
    .context(format!("missing OpenHarmony manifest field {name}"))?;
  if matches.next().is_some() {
    crate::error::bail!("duplicate OpenHarmony manifest field {name}");
  }
  Ok(value)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn config(version: Option<&str>, code: Option<u32>) -> Config {
    let mut config = Config {
      version: version.map(str::to_owned),
      ..Default::default()
    };
    config.bundle.open_harmony.version_code = code;
    config
  }

  #[test]
  fn resolves_config_versions_and_cargo_fallback() {
    let version = AppVersion::from_config(&config(None, None), "1.2.3").unwrap();
    assert_eq!((version.name.as_str(), version.code), ("1.2.3", 1_002_003));
    let version = AppVersion::from_config(&config(Some("2.3.4"), None), "1.2.3").unwrap();
    assert_eq!((version.name.as_str(), version.code), ("2.3.4", 2_003_004));
  }

  #[test]
  fn normalizes_prerelease_and_build_metadata_for_the_sdk() {
    for (input, expected) in [
      ("1.1.5-6", "1.1.5.6"),
      ("1.2.3-rc-1.2+build-7", "1.2.3.rc_1.2_build_7"),
    ] {
      let version = AppVersion::from_config(&config(Some(input), Some(42)), "0.0.0").unwrap();
      assert_eq!((version.name.as_str(), version.code), (expected, 42));
    }
  }

  #[test]
  fn validates_length_codes_and_collision_boundaries() {
    for version in [
      "1.2",
      "1.2.3.4",
      "1.1000.0",
      "1.0.1000",
      "2147.483.648",
      "18446744073709551615.0.0",
    ] {
      assert!(AppVersion::from_config(&config(Some(version), None), "0.0.0").is_err());
    }
    for (version, expected) in [("0.0.0", 0), ("2147.483.647", i32::MAX as u32)] {
      assert_eq!(
        AppVersion::from_config(&config(Some(version), None), "0.0.0")
          .unwrap()
          .code,
        expected
      );
    }
    assert!(AppVersion::from_config(&config(Some("1.2.3"), Some(u32::MAX)), "0.0.0").is_err());
    assert!(
      AppVersion::from_config(&config(Some("1.2.3"), Some(i32::MAX as u32)), "0.0.0").is_ok()
    );
    assert!(AppVersion::from_config(&config(Some("1.1000.0"), Some(7)), "0.0.0").is_ok());
    for (length, accepted) in [(121, true), (122, false)] {
      let version = format!("1.2.3-{}", "a".repeat(length));
      assert_eq!(
        AppVersion::from_config(&config(Some(&version), None), "0.0.0").is_ok(),
        accepted
      );
    }
  }

  #[test]
  fn updates_existing_json5_preserving_comments_and_custom_fields() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("AppScope")).unwrap();
    let path = dir.path().join("AppScope/app.json5");
    fs::write(
      &path,
      r#"// application-owned comment
{app: {bundleName: 'org.example.app', versionName: '1.0.0', /* version comment */
versionCode: 1, vendor: 'example', custom: {versionName: 'untouched', enabled: true}},
other: [1, 2, 3],}"#,
    )
    .unwrap();
    let version = AppVersion::from_config(&config(Some("1.1.5-6"), Some(106)), "0.0.0").unwrap();
    version.synchronize(dir.path()).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    let manifest: serde_json::Value = json5::from_str(&text).unwrap();
    assert_eq!(manifest["app"]["versionName"], "1.1.5.6");
    assert_eq!(manifest["app"]["versionCode"], 106);
    assert_eq!(manifest["app"]["bundleName"], "org.example.app");
    assert_eq!(manifest["app"]["custom"]["versionName"], "untouched");
    assert_eq!(manifest["other"], serde_json::json!([1, 2, 3]));
    assert!(text.contains("application-owned comment") && text.contains("version comment"));
    assert_eq!(
      text,
      r#"// application-owned comment
{app: {bundleName: 'org.example.app', versionName: "1.1.5.6", /* version comment */
versionCode: 106, vendor: 'example', custom: {versionName: 'untouched', enabled: true}},
other: [1, 2, 3],}"#
    );
    version.synchronize(dir.path()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), text);
  }

  #[test]
  fn handles_unicode_escaped_keys_and_either_field_order() {
    let version = AppVersion::from_config(&config(Some("1.2.3"), Some(42)), "0.0.0").unwrap();
    for (input, expected) in [
      (
        r#"{app: {label: '阅读器', 'versionCode': 0x1, 'versionName': 'old',},} // trailing comment"#,
        r#"{app: {label: '阅读器', 'versionCode': 42, 'versionName': "1.2.3",},} // trailing comment"#,
      ),
      (
        r#"{"app": {"version\u004eame": 'old', "version\u0043ode": +1, label: '阅读器'}}"#,
        r#"{"app": {"version\u004eame": "1.2.3", "version\u0043ode": 42, label: '阅读器'}}"#,
      ),
    ] {
      assert_eq!(version.update_manifest(input).unwrap(), expected);
    }
    let equivalent = r#"{app: {versionCode: 0x2a, versionName: '1.2.3'}}"#;
    assert_eq!(version.update_manifest(equivalent).unwrap(), equivalent);
  }

  #[cfg(unix)]
  #[test]
  fn retains_manifest_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("AppScope")).unwrap();
    let path = dir.path().join("AppScope/app.json5");
    fs::write(&path, "{app: {versionName: '1.0.0', versionCode: 1}}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let version = AppVersion::from_config(&config(Some("1.2.3"), None), "0.0.0").unwrap();
    version.synchronize(dir.path()).unwrap();
    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o640
    );
  }

  #[test]
  fn rejects_invalid_manifests_without_rewriting_them() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("AppScope")).unwrap();
    let path = dir.path().join("AppScope/app.json5");
    let version = AppVersion::from_config(&config(Some("1.2.3"), None), "0.0.0").unwrap();
    for input in [
      "{",
      "[]",
      "{app: []}",
      "{app: {versionName: '1.0.0'}}",
      "{app: {versionName: '1.0.0', versionCode: 1, versionCode: 2}}",
      r#"{app: {versionName: '1.0.0', versionCode: 1, "version\u0043ode": 2}}"#,
      "{app: {versionName: '1.0.0', versionCode: 1}, app: {}}",
      "{app: {versionName: {}, versionCode: 1}}",
      "{app: {versionName: '1.0.0', versionCode: []}}",
    ] {
      fs::write(&path, input).unwrap();
      assert!(version.synchronize(dir.path()).is_err(), "{input}");
      assert_eq!(fs::read_to_string(&path).unwrap(), input);
    }
  }
}
