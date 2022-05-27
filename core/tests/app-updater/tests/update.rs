// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports)]

use std::{
  collections::HashMap,
  fs::File,
  path::{Path, PathBuf},
  process::Command,
};

use serde::Serialize;

const UPDATER_PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5YTBGV3JiTy9lRDZVd3NkL0RoQ1htZmExNDd3RmJaNmRMT1ZGVjczWTBKZ0FBQkFBQUFBQUFBQUFBQUlBQUFBQWdMekUzVkE4K0tWQ1hjeGt1Vkx2QnRUR3pzQjVuV0ZpM2czWXNkRm9hVUxrVnB6TUN3K1NheHJMREhQbUVWVFZRK3NIL1VsMDBHNW5ET1EzQno0UStSb21nRW4vZlpTaXIwZFh5ZmRlL1lSN0dKcHdyOUVPclVvdzFhVkxDVnZrbHM2T1o4Tk1NWEU9Cg==";

#[derive(Serialize)]
struct PackageConfig {
  version: &'static str,
}

#[derive(Serialize)]
struct Config {
  package: PackageConfig,
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

fn build_app(cli_bin_path: &Path, cwd: &Path, config: &Config, bundle_updater: bool) {
  let mut command = Command::new(&cli_bin_path);
  command
    .args(["build", "--debug", "--verbose"])
    .arg("--config")
    .arg(serde_json::to_string(config).unwrap())
    .current_dir(&cwd);

  #[cfg(windows)]
  command.args(["--bundles", "msi"]);
  #[cfg(target_os = "linux")]
  command.args(["--bundles", "appimage"]);
  #[cfg(target_os = "macos")]
  command.args(["--bundles", "app"]);

  if bundle_updater {
    command
      .env("TAURI_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
      .env("TAURI_KEY_PASSWORD", "")
      .args(["--bundles", "updater"]);
  }

  let status = command
    .status()
    .expect("failed to run Tauri CLI to bundle app");

  if !status.code().map(|c| c == 0).unwrap_or(true) {
    panic!("failed to bundle app {:?}", status.code());
  }
}

#[cfg(target_os = "linux")]
fn bundle_path(root_dir: &Path, version: &str) -> PathBuf {
  root_dir.join(format!(
    "target/debug/bundle/appimage/app-updater_{}_amd64.AppImage",
    version
  ))
}

#[cfg(target_os = "macos")]
fn bundle_path(root_dir: &Path, _version: &str) -> PathBuf {
  root_dir.join(format!("target/debug/bundle/macos/app-updater.app"))
}

#[cfg(windows)]
fn bundle_path(root_dir: &Path, version: &str) -> PathBuf {
  root_dir.join(format!(
    "target/debug/bundle/msi/app-updater_{}_x64_en-US.msi",
    version
  ))
}

#[test]
#[ignore]
fn update_app() {
  let target = tauri::updater::target().expect("running updater test in an unsupported platform");
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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

  let mut config = Config {
    package: PackageConfig { version: "1.0.0" },
  };

  // bundle app update
  build_app(&cli_bin_path, &manifest_dir, &config, true);

  let updater_ext = if cfg!(windows) { "zip" } else { "tar.gz" };

  let out_bundle_path = bundle_path(&root_dir, "1.0.0");
  let signature_path = out_bundle_path.with_extension(format!(
    "{}.{}.sig",
    out_bundle_path.extension().unwrap().to_str().unwrap(),
    updater_ext
  ));
  let signature = std::fs::read_to_string(&signature_path)
    .unwrap_or_else(|_| panic!("failed to read signature file {}", signature_path.display()));
  let out_updater_path = out_bundle_path.with_extension(format!(
    "{}.{}",
    out_bundle_path.extension().unwrap().to_str().unwrap(),
    updater_ext
  ));
  let updater_path = root_dir.join(format!(
    "target/debug/{}",
    out_updater_path.file_name().unwrap().to_str().unwrap()
  ));
  std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

  std::thread::spawn(move || {
    // start the updater server
    let server = tiny_http::Server::http("localhost:3007").expect("failed to start updater server");

    loop {
      if let Ok(request) = server.recv() {
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
              version: "1.0.0",
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
    }
  });

  config.package.version = "0.1.0";

  // bundle initial app version
  build_app(&cli_bin_path, &manifest_dir, &config, false);

  let mut binary_cmd = if cfg!(windows) {
    Command::new(root_dir.join("target/debug/app-updater.exe"))
  } else if cfg!(target_os = "macos") {
    Command::new(bundle_path(&root_dir, "0.1.0").join("Contents/MacOS/app-updater"))
  } else if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
    let mut c = Command::new("xvfb-run");
    c.arg("--auto-servernum")
      .arg(bundle_path(&root_dir, "0.1.0"));
    c
  } else {
    Command::new(bundle_path(&root_dir, "0.1.0"))
  };

  let status = binary_cmd.status().expect("failed to run app");

  if !status.success() {
    panic!("failed to run app");
  }
}
