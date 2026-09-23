// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;
use colored::Colorize;
use regex::Regex;

use crate::{
  Result, acl,
  error::{Context, ErrorExt},
  helpers::{
    app_paths::{Dirs, resolve_frontend_dir},
    cargo,
    config::Target,
    npm::PackageManager,
  },
};

use serde_json::Value as JsonValue;
use std::{fs, path::Path, process::Command};

#[derive(Debug, Parser)]
#[clap(about = "Add a tauri plugin to the project")]
pub struct Options {
  /// The plugin to add.
  pub plugin: String,
  /// Git tag to use.
  #[clap(short, long)]
  pub tag: Option<String>,
  /// Git rev to use.
  #[clap(short, long)]
  pub rev: Option<String>,
  /// Git branch to use.
  #[clap(short, long)]
  pub branch: Option<String>,
  /// Don't format code with rustfmt
  #[clap(long)]
  pub no_fmt: bool,
}

pub fn command(options: Options) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  run(options, &dirs)
}

pub fn run(options: Options, dirs: &Dirs) -> Result<()> {
  let (plugin, version) = options
    .plugin
    .split_once('@')
    .map(|(p, v)| (p, Some(v)))
    .unwrap_or((&options.plugin, None));

  let mut plugins = crate::helpers::plugins::known_plugins();
  let (metadata, is_known) = plugins
    .remove(plugin)
    .map(|metadata| (metadata, true))
    .unwrap_or_default();

  let plugin_snake_case = plugin.replace('-', "_");
  let crate_name = format!("tauri-plugin-{plugin}");
  let npm_name = if is_known {
    format!("@tauri-apps/plugin-{plugin}")
  } else {
    format!("tauri-plugin-{plugin}-api")
  };

  let git_ref = git_ref(
    options.tag.as_deref(),
    options.rev.as_deref(),
    options.branch.as_deref(),
  )?;

  if git_ref.is_some() {
    if !is_known {
      crate::error::bail!(
        "Git options --tag, --rev and --branch can only be used with official Tauri plugins"
      );
    }
    if version.is_some() {
      crate::error::bail!("A version cannot be used together with --tag, --rev or --branch");
    }
  }

  let frontend_dir = resolve_frontend_dir();

  let target_str = metadata
    .desktop_only
    .then_some(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#)
    .or_else(|| {
      metadata
        .mobile_only
        .then_some(r#"cfg(any(target_os = "android", target_os = "ios"))"#)
    });

  // a git dependency cannot also have a registry version requirement
  let cargo_version_req = if git_ref.is_some() {
    None
  } else {
    version.or(metadata.version_req.as_deref())
  };

  cargo::install_one(cargo::CargoInstallOptions {
    name: &crate_name,
    version: cargo_version_req,
    branch: options.branch.as_deref(),
    rev: options.rev.as_deref(),
    tag: options.tag.as_deref(),
    cwd: Some(dirs.tauri),
    target: target_str,
  })?;

  if !metadata.rust_only {
    if let Some(manager) = frontend_dir.map(PackageManager::from_project) {
      let npm_spec = if let Some(git_ref) = git_ref {
        format!("tauri-apps/tauri-plugin-{plugin}#{git_ref}")
      } else if let Some(version_req) = version
        .map(ToString::to_string)
        .or(metadata.version_req.as_ref().map(|v| format!("~{v}")))
      {
        format!("{npm_name}@{version_req}")
      } else {
        npm_name
      };
      manager.install(&[npm_spec], dirs.frontend)?;
    }

    let _ = acl::permission::add::command(acl::permission::add::Options {
      identifier: format!("{plugin}:default"),
      capability: None,
    });
  }

  if plugin == "updater" {
    // not fatal: the plugin is installed by this point, and the developer can set the option by
    // hand if we could not
    if let Err(e) = enable_require_signed_version(dirs.tauri) {
      log::warn!(
        "Failed to enable `plugins > updater > requireSignedVersion`, set it manually: {e}"
      );
    }
  }

  // add plugin init code to main.rs or lib.rs
  let plugin_init_fn = if plugin == "stronghold" {
    "Builder::new(|pass| todo!()).build()"
  } else if plugin == "localhost" {
    "Builder::new(todo!()).build()"
  } else if plugin == "single-instance" {
    "init(|app, args, cwd| {})"
  } else if plugin == "log" {
    "Builder::new().level(tauri_plugin_log::log::LevelFilter::Info).build()"
  } else if metadata.builder {
    "Builder::new().build()"
  } else {
    "init()"
  };
  let plugin_init = format!(".plugin(tauri_plugin_{plugin_snake_case}::{plugin_init_fn})");

  let re = Regex::new(r"(tauri\s*::\s*Builder\s*::\s*default\(\))(\s*)").unwrap();
  for file in [
    dirs.tauri.join("src/main.rs"),
    dirs.tauri.join("src/lib.rs"),
  ] {
    let contents =
      std::fs::read_to_string(&file).fs_context("failed to read Rust entry point", file.clone())?;

    if contents.contains(&plugin_init) {
      log::info!(
        "Plugin initialization code already found on {}",
        file.display()
      );
      return Ok(());
    }

    if re.is_match(&contents) {
      let out = re.replace(&contents, format!("$1$2{plugin_init}$2"));

      log::info!("Adding plugin to {}", file.display());
      std::fs::write(&file, out.as_bytes()).fs_context("failed to write plugin init code", file)?;

      if !options.no_fmt {
        // reformat code with rustfmt
        log::info!("Running `cargo fmt`...");
        let _ = Command::new("cargo")
          .arg("fmt")
          .current_dir(dirs.tauri)
          .status();
      }

      return Ok(());
    }
  }

  let builder_code = if metadata.builder {
    format!(r#"+    .plugin(tauri_plugin_{plugin_snake_case}::Builder::new().build())"#,)
  } else {
    format!(r#"+    .plugin(tauri_plugin_{plugin_snake_case}::init())"#)
  };

  let rust_code = format!(
    r#" {}
{}
     {}"#,
    "tauri::Builder::default()".dimmed(),
    builder_code.normal().green(),
    r#".invoke_handler(tauri::generate_handler![])
     .run(tauri::generate_context!())
     .expect("error while running tauri application");"#
      .dimmed(),
  );

  log::warn!(
    "Couldn't find `{}` in `{}` or `{}`, you must enable the plugin in your Rust code manually:\n\n{}",
    "tauri::Builder".cyan(),
    "main.rs".cyan(),
    "lib.rs".cyan(),
    rust_code
  );

  Ok(())
}

/// Returns the single git ref given with `--tag`, `--rev` or `--branch`, if any.
fn git_ref<'a>(
  tag: Option<&'a str>,
  rev: Option<&'a str>,
  branch: Option<&'a str>,
) -> Result<Option<&'a str>> {
  match (tag, rev, branch) {
    (Some(r), None, None) | (None, Some(r), None) | (None, None, Some(r)) => Ok(Some(r)),
    (None, None, None) => Ok(None),
    _ => crate::error::bail!("Only one of --tag, --rev and --branch can be specified"),
  }
}

/// Turns on the updater's signed version check for a project adopting the plugin now.
///
/// An update endpoint response is not signed, so the version it announces does not by itself
/// prove which release its `url` and `signature` point at. `requireSignedVersion` makes the
/// updater compare that version against the one recorded inside the signature, which is what
/// stops a tampered response from pairing a new version number with an older release to force a
/// downgrade.
///
/// This can only be the default for a project adding the plugin now, because the check also
/// rejects releases that were signed before the version was recorded. A project already serving
/// such releases has to re-sign them first, so a value that is already in the config is left
/// alone.
fn enable_require_signed_version(tauri_dir: &Path) -> Result<()> {
  let (mut config, config_path) =
    tauri_utils::config::parse::parse_value(Target::current(), tauri_dir.join("tauri.conf.json"))
      .context("failed to parse config")?;

  if !set_require_signed_version(&mut config) {
    return Ok(());
  }

  let serialized = if config_path.extension().is_some_and(|ext| ext == "toml") {
    toml::to_string_pretty(&config).context("failed to serialize config")?
  } else {
    serde_json::to_string_pretty(&config).context("failed to serialize config")?
  };
  fs::write(&config_path, serialized).fs_context("failed to write config", config_path.clone())?;

  log::info!(
    "Enabled `plugins > updater > requireSignedVersion` in {}",
    config_path.display()
  );
  log::info!("Remember to set `plugins > updater > pubkey` and `endpoints`.");

  Ok(())
}

/// Sets `plugins > updater > requireSignedVersion`, returning whether the config changed.
fn set_require_signed_version(config: &mut JsonValue) -> bool {
  let Some(root) = config.as_object_mut() else {
    return false;
  };
  let plugins = root
    .entry("plugins")
    .or_insert_with(|| JsonValue::Object(Default::default()));
  let Some(plugins) = plugins.as_object_mut() else {
    return false;
  };
  let updater = plugins
    .entry("updater")
    .or_insert_with(|| JsonValue::Object(Default::default()));
  let Some(updater) = updater.as_object_mut() else {
    return false;
  };

  // a value already in the config is a deliberate choice, an explicit `false` included
  if updater.contains_key("requireSignedVersion") || updater.contains_key("require-signed-version")
  {
    return false;
  }

  updater.insert("requireSignedVersion".into(), JsonValue::Bool(true));
  true
}

#[cfg(test)]
mod tests {
  use super::{git_ref, set_require_signed_version};
  use serde_json::json;

  #[test]
  fn enables_required_version_signing_on_a_config_without_updater_plugin_config() {
    let mut config = json!({ "identifier": "com.tauri.dev" });
    assert!(set_require_signed_version(&mut config));
    assert_eq!(config["plugins"]["updater"]["requireSignedVersion"], true);
    // the rest of the config is carried through untouched
    assert_eq!(config["identifier"], "com.tauri.dev");
  }

  #[test]
  fn keeps_the_existing_updater_config() {
    let mut config = json!({ "plugins": { "updater": { "pubkey": "abc" } } });
    assert!(set_require_signed_version(&mut config));
    assert_eq!(config["plugins"]["updater"]["pubkey"], "abc");
    assert_eq!(config["plugins"]["updater"]["requireSignedVersion"], true);
  }

  #[test]
  fn never_overwrites_a_deliberate_choice() {
    // a project that still serves releases signed before the version was recorded has to keep
    // this off until it has re-signed them, so an explicit `false` must survive
    for existing in ["requireSignedVersion", "require-signed-version"] {
      let mut config = json!({ "plugins": { "updater": { existing: false } } });
      assert!(!set_require_signed_version(&mut config));
      assert_eq!(config["plugins"]["updater"][existing], false);
      assert!(
        config["plugins"]["updater"]
          .as_object()
          .is_some_and(|updater| updater.len() == 1)
      );
    }
  }

  #[test]
  fn leaves_a_config_it_cannot_understand_alone() {
    let mut config = json!({ "plugins": { "updater": "not an object" } });
    assert!(!set_require_signed_version(&mut config));
    assert_eq!(config["plugins"]["updater"], "not an object");
  }

  #[test]
  fn accepts_at_most_one_git_ref() {
    assert_eq!(git_ref(None, None, None).unwrap(), None);
    assert_eq!(git_ref(Some("v2"), None, None).unwrap(), Some("v2"));
    assert_eq!(git_ref(None, Some("abc"), None).unwrap(), Some("abc"));
    assert_eq!(git_ref(None, None, Some("dev")).unwrap(), Some("dev"));
    assert!(git_ref(Some("v2"), Some("abc"), None).is_err());
    assert!(git_ref(Some("v2"), None, Some("dev")).is_err());
    assert!(git_ref(None, Some("abc"), Some("dev")).is_err());
  }
}
