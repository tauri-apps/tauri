// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports)]

use std::{
  collections::HashMap,
  fs::File,
  path::{Path, PathBuf},
  process::Command,
  sync::Arc,
};

use serde::Serialize;

const UPDATER_PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5dkpDN09RZm5GeVAzc2RuYlNzWVVJelJRQnNIV2JUcGVXZUplWXZXYXpqUUFBQkFBQUFBQUFBQUFBQUlBQUFBQTZrN2RnWGh5dURxSzZiL1ZQSDdNcktiaHRxczQwMXdQelRHbjRNcGVlY1BLMTBxR2dpa3I3dDE1UTVDRDE4MXR4WlQwa1BQaXdxKy9UU2J2QmVSNXhOQWFDeG1GSVllbUNpTGJQRkhhTnROR3I5RmdUZi90OGtvaGhJS1ZTcjdZU0NyYzhQWlQ5cGM9Cg==";
// const UPDATER_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEZFOUJFNDg1NTU4NUZDQUQKUldTdC9JVlZoZVNiL2tVVG1hSFRETjRIZXE0a0F6d3dSY2ViYzdrSFh2MjBGWm1jM0NoWVFqM1YK";

const UPDATER_PRIVATE_KEY_NEXT: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5OUxRK2FpTzVPQWt6M2laMWNodDI5QnJEL1Y2Z3pjREprTW9TMkc1Z1BuWUFBQkFBQUFBQUFBQUFBQUlBQUFBQVFCTkRHdHZlLzRTbHIxSUNXdFY0VnZaODhLdGExa1B4R240UGdqekFRcVNDd2xkeDMvZkFZZTJEYUxqSE5BZnc2Sk5VNGdmU0Y0Nml3QU92WWRaRlFGUUtaZWNSMWxjaisyc1pZSUk0RXB1N3BrbXlSYitZMHR0MEVsOUdxZk56eEZoZ0diUXRXLzg9Cg==";
const UPDATER_PUBLIC_KEY_NEXT: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDg5Nzg5MDdEREM5MDNBRjkKUldUNU9wRGNmWkI0aWZtY0toN3lxeHRJMmJ5bjFWdit6eXB2QWtJMVhzYjdrc3VIdDQxVVFwMVQK";

const UPDATED_EXIT_CODE: i32 = 0;
const UP_TO_DATE_EXIT_CODE: i32 = 2;
const UPDATE_APP_VERSION: &str = "1.0.0";

#[derive(Serialize)]
struct PackageConfig {
  version: &'static str,
}

#[derive(Serialize)]
struct UpdaterConfig {
  pubkey: &'static str,
}

#[derive(Serialize)]
struct TauriConfig {
  updater: UpdaterConfig,
}

#[derive(Serialize)]
struct Config {
  package: PackageConfig,
  tauri: TauriConfig,
}

#[derive(Serialize)]
struct ConfigV2 {
  version: &'static str,
  bundle: BundleConfig,
  plugins: PluginsConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleConfig {
  pub create_updater_artifacts: Option<bool>,
}

#[derive(Serialize)]
struct PluginsConfig {
  updater: UpdaterConfig,
}

#[derive(Serialize)]
struct PlatformUpdate {
  signature: String,
  url: &'static str,
  with_elevated_task: bool,
}

#[derive(Serialize)]
struct Update {
  version: &'static str,
  date: String,
  platforms: HashMap<String, PlatformUpdate>,
}

fn get_cli_bin_path(cli_dir: &Path, debug: bool) -> Option<PathBuf> {
  let mut cli_bin_path = cli_dir.join(format!(
    "target/{}/cargo-tauri",
    if debug { "debug" } else { "release" }
  ));
  if cfg!(windows) {
    cli_bin_path.set_extension("exe");
  }
  if cli_bin_path.exists() {
    Some(cli_bin_path)
  } else {
    None
  }
}

fn build_app(
  cli_bin_path: &Path,
  cwd: &Path,
  envs: Vec<(&str, &str)>,
  config: &Config,
  bundle_updater: bool,
  target: BundleTarget,
) {
  let mut command = Command::new(cli_bin_path);
  command
    .args(["build", "--debug", "--verbose"])
    .arg("--config")
    .arg(serde_json::to_string(config).unwrap())
    .current_dir(cwd);

  #[cfg(target_os = "linux")]
  command.args(["--bundles", target.name()]);
  #[cfg(target_os = "macos")]
  command.args(["--bundles", target.name()]);

  if bundle_updater {
    #[cfg(windows)]
    command.args(["--bundles", "msi", "nsis"]);

    command
      .envs(envs)
      .env("TAURI_KEY_PASSWORD", "")
      .args(["--bundles", "updater"]);
  } else {
    #[cfg(windows)]
    command.args(["--bundles", target.name()]);
  }

  let status = command
    .status()
    .expect("failed to run Tauri CLI to bundle app");

  if !status.code().map(|c| c == 0).unwrap_or(true) {
    panic!("failed to bundle app {:?}", status.code());
  }
}

fn build_app_v2(cwd: &Path, envs: Vec<(&str, &str)>, config: &ConfigV2, target: BundleTarget) {
  let mut command = cross_command("npm");
  command
    .args(["run", "tauri", "--", "build", "--debug", "-vv"])
    .arg("--config")
    .arg(serde_json::to_string(config).unwrap())
    .current_dir(cwd);

  #[cfg(target_os = "linux")]
  command.args(["--bundles", target.name()]);
  #[cfg(target_os = "macos")]
  command.args(["--bundles", target.name()]);

  if config.bundle.create_updater_artifacts.map_or(false, |c| c) {
    #[cfg(windows)]
    command.args(["--bundles", "msi", "nsis"]);

    command
      .envs(envs)
      .env("TAURI_SIGNING_KEY_PASSWORD", "")
      .env("CI", "true")
      // skip password prompt
      .args(["--bundles", "updater"]);
  } else {
    #[cfg(windows)]
    command.args(["--bundles", target.name()]);
  }

  let status = command
    .status()
    .expect("failed to run Tauri CLI to bundle app");

  if !status.code().map(|c| c == 0).unwrap_or(true) {
    panic!("failed to bundle app {:?}", status.code());
  }
}

#[derive(Copy, Clone)]
enum BundleTarget {
  AppImage,

  App,

  Msi,
  Nsis,
}

impl BundleTarget {
  fn name(self) -> &'static str {
    match self {
      Self::AppImage => "appimage",
      Self::App => "app",
      Self::Msi => "msi",
      Self::Nsis => "nsis",
    }
  }
}

impl Default for BundleTarget {
  fn default() -> Self {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    return Self::App;
    #[cfg(target_os = "linux")]
    return Self::AppImage;
    #[cfg(windows)]
    return Self::Nsis;
  }
}

#[cfg(target_os = "linux")]
fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
  vec![(
    BundleTarget::AppImage,
    root_dir.join(format!(
      "target/debug/bundle/appimage/app-updater_{version}_amd64.AppImage"
    )),
  )]
}

#[cfg(target_os = "macos")]
fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
  vec![(
    BundleTarget::App,
    root_dir.join("target/debug/bundle/macos/app-updater.app"),
  )]
}

#[cfg(target_os = "ios")]
fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
  vec![(
    BundleTarget::App,
    root_dir.join("target/debug/bundle/ios/app-updater.app"),
  )]
}

#[cfg(windows)]
fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
  vec![
    (
      BundleTarget::Nsis,
      root_dir.join(format!(
        "target/debug/bundle/nsis/app-updater_{version}_x64-setup.exe"
      )),
    ),
    (
      BundleTarget::Msi,
      root_dir.join(format!(
        "target/debug/bundle/msi/app-updater_{version}_x64_en-US.msi"
      )),
    ),
  ]
}

enum TauriVersion {
  V1,
  V2,
}

#[test]
#[serial_test::serial(updater)]
#[ignore]
fn update_app_v1() {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let fixture_dir = manifest_dir.join("tests/fixtures/tauri-v1");

  update_app_flow(|options| {
    let config = Config {
      package: PackageConfig {
        version: UPDATE_APP_VERSION,
      },
      tauri: TauriConfig {
        updater: UpdaterConfig {
          pubkey: UPDATER_PUBLIC_KEY_NEXT,
        },
      },
    };
    build_app(
      &options.cli_bin_path,
      &fixture_dir,
      vec![("TAURI_PRIVATE_KEY", UPDATER_PRIVATE_KEY_NEXT)],
      &config,
      true,
      Default::default(),
    );

    (fixture_dir, TauriVersion::V1)
  });
}

fn cross_command(bin: &str) -> Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = Command::new(bin);
  cmd
}

#[test]
#[serial_test::serial(updater)]
#[ignore]
fn update_app_to_v2() {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let fixture_dir = manifest_dir.join("tests/fixtures/tauri-v2/src-tauri");

  update_app_flow(|_options| {
    // npm install
    let status = cross_command("npm")
      .arg("install")
      .current_dir(&fixture_dir)
      .status()
      .expect("failed to install v2 dependencies");
    if !status.success() {
      panic!("failed to install v2 dependencies");
    }

    let config = ConfigV2 {
      version: UPDATE_APP_VERSION,
      plugins: PluginsConfig {
        updater: UpdaterConfig {
          pubkey: UPDATER_PUBLIC_KEY_NEXT,
        },
      },
      bundle: BundleConfig {
        create_updater_artifacts: Some(true),
      },
    };
    build_app_v2(
      &fixture_dir,
      vec![("TAURI_SIGNING_PRIVATE_KEY", UPDATER_PRIVATE_KEY_NEXT)],
      &config,
      Default::default(),
    );

    (fixture_dir, TauriVersion::V2)
  });
}

struct Options<'a> {
  cli_bin_path: &'a Path,
}

fn update_app_flow<F: FnOnce(Options<'_>) -> (PathBuf, TauriVersion)>(build_app_updater: F) {
  let target = tauri::updater::target().expect("running updater test in an unsupported platform");
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let tauri_v1_fixture_dir = manifest_dir.join("tests/fixtures/tauri-v1");
  let root_dir = manifest_dir.join("../../..");
  let cli_dir = root_dir.join("tooling/cli");

  let cli_bin_path = if let Some(p) = get_cli_bin_path(&cli_dir, false) {
    p
  } else if let Some(p) = get_cli_bin_path(&cli_dir, true) {
    p
  } else {
    let status = Command::new("cargo")
      .arg("build")
      .current_dir(&cli_dir)
      .status()
      .expect("failed to run cargo");
    if !status.success() {
      panic!("failed to build CLI");
    }
    get_cli_bin_path(&cli_dir, true).expect("cargo did not build the Tauri CLI")
  };

  // bundle app update
  let (app_root, tauri_version) = build_app_updater(Options {
    cli_bin_path: &cli_bin_path,
  });

  let updater_zip_ext = match tauri_version {
    TauriVersion::V1 => Some(if cfg!(windows) { "zip" } else { "tar.gz" }),
    TauriVersion::V2 => {
      if cfg!(target_os = "macos") {
        Some("tar.gz")
      } else {
        None
      }
    }
  };

  for (bundle_target, out_bundle_path) in bundle_paths(&app_root, UPDATE_APP_VERSION) {
    let mut bundle_updater_ext = out_bundle_path
      .extension()
      .unwrap()
      .to_str()
      .unwrap()
      .to_string();
    if matches!(tauri_version, TauriVersion::V1) {
      bundle_updater_ext = bundle_updater_ext.replace("exe", "nsis");
    }

    let (out_updater_path, signature) = if let Some(updater_zip_ext) = &updater_zip_ext {
      let signature_path =
        out_bundle_path.with_extension(format!("{bundle_updater_ext}.{updater_zip_ext}.sig"));
      let signature = std::fs::read_to_string(&signature_path)
        .unwrap_or_else(|_| panic!("failed to read signature file {}", signature_path.display()));

      let out_updater_path =
        out_bundle_path.with_extension(format!("{}.{}", bundle_updater_ext, updater_zip_ext));

      (out_updater_path, signature)
    } else {
      let signature_path = out_bundle_path.with_extension(format!("{bundle_updater_ext}.sig"));
      let signature = std::fs::read_to_string(&signature_path)
        .unwrap_or_else(|_| panic!("failed to read signature file {}", signature_path.display()));

      (out_bundle_path, signature)
    };

    let updater_path = app_root.join(format!(
      "target/debug/{}",
      out_updater_path.file_name().unwrap().to_str().unwrap()
    ));
    std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

    let target = target.clone();

    // create the updater server
    let server =
      Arc::new(tiny_http::Server::http("localhost:3007").expect("failed to start updater server"));

    let server_ = server.clone();
    std::thread::spawn(move || {
      for request in server_.incoming_requests() {
        match request.url() {
          "/" => {
            let mut platforms = HashMap::new();

            platforms.insert(
              target.clone(),
              PlatformUpdate {
                signature: signature.clone(),
                url: "http://localhost:3007/download",
                with_elevated_task: false,
              },
            );
            let body = serde_json::to_vec(&Update {
              version: UPDATE_APP_VERSION,
              date: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
              platforms,
            })
            .unwrap();
            let len = body.len();
            let response = tiny_http::Response::new(
              tiny_http::StatusCode(200),
              Vec::new(),
              std::io::Cursor::new(body),
              Some(len),
              None,
            );
            let _ = request.respond(response);
          }
          "/download" => {
            let _ = request.respond(tiny_http::Response::from_file(
              File::open(&updater_path).unwrap_or_else(|_| {
                panic!("failed to open updater bundle {}", updater_path.display())
              }),
            ));
          }
          _ => (),
        }
      }
    });

    let config = Config {
      package: PackageConfig { version: "0.1.0" },
      tauri: TauriConfig {
        updater: UpdaterConfig {
          pubkey: UPDATER_PUBLIC_KEY_NEXT,
        },
      },
    };

    // bundle initial app version
    build_app(
      &cli_bin_path,
      &tauri_v1_fixture_dir,
      vec![("TAURI_PRIVATE_KEY", UPDATER_PRIVATE_KEY)],
      &config,
      false,
      bundle_target,
    );

    let status_checks = if matches!(bundle_target, BundleTarget::Msi) {
      // for msi we can't really check if the app was updated, because we can't change the install path
      vec![UPDATED_EXIT_CODE]
    } else {
      vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE]
    };

    for expected_exit_code in status_checks {
      let mut binary_cmd = if cfg!(windows) {
        Command::new(tauri_v1_fixture_dir.join("target/debug/app-updater.exe"))
      } else if cfg!(target_os = "macos") {
        Command::new(
          bundle_paths(&tauri_v1_fixture_dir, "0.1.0")
            .first()
            .unwrap()
            .1
            .join("Contents/MacOS/app-updater"),
        )
      } else if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
        let mut c = Command::new("xvfb-run");
        c.arg("--auto-servernum").arg(
          &bundle_paths(&tauri_v1_fixture_dir, "0.1.0")
            .first()
            .unwrap()
            .1,
        );
        c
      } else {
        Command::new(
          &bundle_paths(&tauri_v1_fixture_dir, "0.1.0")
            .first()
            .unwrap()
            .1,
        )
      };

      binary_cmd.env("TARGET", bundle_target.name());

      let status = binary_cmd.status().expect("failed to run app");

      // Verify the framework extracted symlinks correctly
      #[cfg(target_os = "macos")]
      {
        let meta = std::fs::symlink_metadata(
          bundle_paths(&tauri_v1_fixture_dir, "0.1.0")
            .first()
            .unwrap()
            .1
            .join("Contents/Frameworks/test.framework/Modules"),
        )
        .expect("test.framework/Modules metadata");
        assert!(
          meta.file_type().is_symlink(),
          "test.framework/Modules should be a symlink"
        );
      }

      let code = status.code().unwrap_or(-1);
      if code != expected_exit_code {
        panic!("failed to update app\nexpected {expected_exit_code} got {code}",);
      }

      // wait for the update to be applied on Windows
      #[cfg(windows)]
      std::thread::sleep(std::time::Duration::from_secs(3));
    }

    // force Rust to rebuild the binary so it doesn't conflict with other test runs
    #[cfg(windows)]
    std::fs::remove_file(tauri_v1_fixture_dir.join("target/debug/app-updater.exe")).unwrap();

    // graceful shutdown
    server.unblock();
  }
}
