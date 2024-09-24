// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Result};
use cargo_toml::{Dependency, Manifest};
use tauri_utils::config::{AppConfig, Config, PatternKind};

#[derive(Debug, Default, PartialEq, Eq)]
struct Diff {
  remove: Vec<String>,
  add: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum DependencyKind {
  Build,
  Normal,
}

#[derive(Debug)]
struct AllowlistedDependency {
  name: String,
  alias: Option<String>,
  kind: DependencyKind,
  all_cli_managed_features: Option<Vec<&'static str>>,
  expected_features: Vec<String>,
}

pub fn check(config: &Config, manifest: &mut Manifest) -> Result<()> {
  let dependencies = vec![
    AllowlistedDependency {
      name: "tauri-build".into(),
      alias: None,
      kind: DependencyKind::Build,
      all_cli_managed_features: Some(vec!["isolation"]),
      expected_features: match config.app.security.pattern {
        PatternKind::Isolation { .. } => vec!["isolation".to_string()],
        _ => vec![],
      },
    },
    AllowlistedDependency {
      name: "tauri".into(),
      alias: None,
      kind: DependencyKind::Normal,
      all_cli_managed_features: Some(
        AppConfig::all_features()
          .into_iter()
          .filter(|f| f != &"tray-icon")
          .collect(),
      ),
      expected_features: config
        .app
        .features()
        .into_iter()
        .filter(|f| f != &"tray-icon")
        .map(|f| f.to_string())
        .collect::<Vec<String>>(),
    },
  ];

  for metadata in dependencies {
    let mut name = metadata.name.clone();
    let mut deps = find_dependency(manifest, &metadata.name, metadata.kind);
    if deps.is_empty() {
      if let Some(alias) = &metadata.alias {
        deps = find_dependency(manifest, alias, metadata.kind);
        name.clone_from(alias);
      }
    }

    for dep in deps {
      if let Err(error) = check_features(dep, &metadata) {
        return Err(anyhow!("
      The `{}` dependency features on the `Cargo.toml` file does not match the allowlist defined under `tauri.conf.json`.
      Please run `tauri dev` or `tauri build` or {}.
    ", name, error));
      }
    }
  }

  Ok(())
}

fn find_dependency(manifest: &mut Manifest, name: &str, kind: DependencyKind) -> Vec<Dependency> {
  let dep = match kind {
    DependencyKind::Build => manifest.build_dependencies.remove(name),
    DependencyKind::Normal => manifest.dependencies.remove(name),
  };

  if let Some(dep) = dep {
    vec![dep]
  } else {
    let mut deps = Vec::new();
    for target in manifest.target.values_mut() {
      if let Some(dep) = match kind {
        DependencyKind::Build => target.build_dependencies.remove(name),
        DependencyKind::Normal => target.dependencies.remove(name),
      } {
        deps.push(dep);
      }
    }
    deps
  }
}

fn features_diff(current: &[String], expected: &[String]) -> Diff {
  let mut remove = Vec::new();
  let mut add = Vec::new();
  for feature in current {
    if !expected.contains(feature) {
      remove.push(feature.clone());
    }
  }

  for feature in expected {
    if !current.contains(feature) {
      add.push(feature.clone());
    }
  }

  Diff { remove, add }
}

fn check_features(dependency: Dependency, metadata: &AllowlistedDependency) -> Result<(), String> {
  let features = match dependency {
    Dependency::Simple(_) => Vec::new(),
    Dependency::Detailed(dep) => dep.features,
    Dependency::Inherited(dep) => dep.features,
  };

  let diff = if let Some(all_cli_managed_features) = &metadata.all_cli_managed_features {
    features_diff(
      &features
        .into_iter()
        .filter(|f| all_cli_managed_features.contains(&f.as_str()))
        .collect::<Vec<String>>(),
      &metadata.expected_features,
    )
  } else {
    features_diff(
      &features
        .into_iter()
        .filter(|f| f.starts_with("allow-"))
        .collect::<Vec<String>>(),
      &metadata.expected_features,
    )
  };

  let mut error_message = String::new();
  if !diff.remove.is_empty() {
    error_message.push_str("remove the `");
    error_message.push_str(&diff.remove.join(", "));
    error_message.push_str(if diff.remove.len() == 1 {
      "` feature"
    } else {
      "` features"
    });
    if !diff.add.is_empty() {
      error_message.push_str(" and ");
    }
  }
  if !diff.add.is_empty() {
    error_message.push_str("add the `");
    error_message.push_str(&diff.add.join(", "));
    error_message.push_str(if diff.add.len() == 1 {
      "` feature"
    } else {
      "` features"
    });
  }

  if error_message.is_empty() {
    Ok(())
  } else {
    Err(error_message)
  }
}

#[cfg(test)]
mod tests {
  use super::Diff;

  #[test]
  fn array_diff() {
    for (current, expected, result) in [
      (vec![], vec![], Default::default()),
      (
        vec!["a".into()],
        vec![],
        Diff {
          remove: vec!["a".into()],
          add: vec![],
        },
      ),
      (vec!["a".into()], vec!["a".into()], Default::default()),
      (
        vec!["a".into(), "b".into()],
        vec!["a".into()],
        Diff {
          remove: vec!["b".into()],
          add: vec![],
        },
      ),
      (
        vec!["a".into(), "b".into()],
        vec!["a".into(), "c".into()],
        Diff {
          remove: vec!["b".into()],
          add: vec!["c".into()],
        },
      ),
    ] {
      assert_eq!(crate::manifest::features_diff(&current, &expected), result);
    }
  }
}
