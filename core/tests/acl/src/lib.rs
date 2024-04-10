// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {
  use std::{
    collections::BTreeMap,
    env::temp_dir,
    fs::{read_dir, read_to_string},
    path::Path,
  };

  use tauri_utils::{
    acl::{build::parse_capabilities, manifest::Manifest, resolved::Resolved},
    platform::Target,
  };

  fn load_plugins(plugins: &[String]) -> BTreeMap<String, Manifest> {
    let mut manifests = BTreeMap::new();

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = temp_dir();

    for plugin in plugins {
      let plugin_path = manifest_dir.join("fixtures").join("plugins").join(plugin);

      let permission_files = tauri_utils::acl::build::define_permissions(
        &format!("{}/*.toml", plugin_path.display()),
        plugin,
        &out_dir,
        |_| true,
      )
      .expect("failed to define permissions");
      let manifest = Manifest::new(permission_files, None);
      manifests.insert(plugin.to_string(), manifest);
    }

    manifests
  }

  #[test]
  fn resolve_acl() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_path = manifest_dir.join("fixtures").join("capabilities");
    for fixture_path in read_dir(fixtures_path).expect("failed to read fixtures") {
      let fixture_entry = fixture_path.expect("failed to read fixture entry");

      let mut settings = insta::Settings::clone_current();
      settings.set_snapshot_path(
        if fixture_entry.path().file_name().unwrap() == "platform-specific-permissions" {
          Path::new("../fixtures/snapshots").join(Target::current().to_string())
        } else {
          Path::new("../fixtures/snapshots").to_path_buf()
        },
      );
      let _guard = settings.bind_to_scope();

      let fixture_plugins_str = read_to_string(fixture_entry.path().join("required-plugins.json"))
        .expect("failed to read fixture required-plugins.json file");
      let fixture_plugins: Vec<String> = serde_json::from_str(&fixture_plugins_str)
        .expect("required-plugins.json is not a valid JSON");

      let manifests = load_plugins(&fixture_plugins);
      let capabilities = parse_capabilities(&format!("{}/cap*", fixture_entry.path().display()))
        .expect("failed to parse capabilities");

      let resolved = Resolved::resolve(&manifests, capabilities, Target::current())
        .expect("failed to resolve ACL");

      insta::assert_debug_snapshot!(
        fixture_entry
          .path()
          .file_name()
          .unwrap()
          .to_string_lossy()
          .to_string(),
        resolved
      );
    }
  }
}
