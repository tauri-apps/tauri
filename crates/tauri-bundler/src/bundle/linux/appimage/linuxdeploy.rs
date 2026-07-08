// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{super::debian, write_and_make_executable};
use crate::{
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::{
    fs_utils,
    http_utils::{download_and_verify, verify_file_hash, HashAlgorithm},
    CommandExt,
  },
  Settings,
};
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the AppImage was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the deb binary name
  let appimage_arch: &str = match settings.binary_arch() {
    Arch::X86_64 => "amd64",
    Arch::X86 => "i386",
    Arch::AArch64 => "aarch64",
    Arch::Armhf => "armhf",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };

  let tools_arch = if settings.binary_arch() == Arch::Armhf {
    "armhf"
  } else {
    settings.target().split('-').next().unwrap()
  };

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }

  let tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| output_path.to_path_buf(), |p| p.join("tauri"))
    });

  fs::create_dir_all(&tools_path)?;

  let linuxdeploy_path = prepare_tools(
    &tools_path,
    tools_arch,
    settings.log_level() != log::Level::Error,
  )?;

  let package_dir = settings.project_out_directory().join("bundle/appimage_deb");

  let main_binary = settings.main_binary()?;
  let product_name = settings.product_name();

  let mut settings = settings.clone();
  if main_binary.name().contains(' ') {
    let main_binary_path = settings.binary_path(main_binary);
    let project_out_directory = settings.project_out_directory();

    let main_binary_name_kebab = heck::AsKebabCase(main_binary.name()).to_string();
    let new_path = project_out_directory.join(&main_binary_name_kebab);
    fs::copy(main_binary_path, new_path)?;

    let main_binary = settings.main_binary_mut()?;
    main_binary.set_name(main_binary_name_kebab);
  }

  // generate deb_folder structure
  let (data_dir, icons) = debian::generate_data(&settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.appimage().files, &data_dir)
    .with_context(|| "Failed to copy custom files")?;

  fs::create_dir_all(&output_path)?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.product_name()));
  let appimage_filename = format!(
    "{}_{}_{}.AppImage",
    settings.product_name(),
    settings.version_string(),
    appimage_arch
  );
  let appimage_path = output_path.join(&appimage_filename);
  fs_utils::create_dir(&app_dir_path, true)?;

  fs::create_dir_all(&tools_path)?;
  let larger_icon = icons
    .iter()
    .filter(|i| i.width == i.height)
    .max_by_key(|i| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");
  let larger_icon_path = larger_icon
    .path
    .strip_prefix(package_dir.join("data"))
    .unwrap()
    .to_string_lossy()
    .to_string();

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  let app_dir_usr = app_dir_path.join("usr/");
  let app_dir_usr_bin = app_dir_usr.join("bin/");
  let app_dir_usr_lib = app_dir_usr.join("lib/");

  fs_utils::copy_dir(&data_dir.join("usr/"), &app_dir_usr)?;

  // Using create_dir_all for a single dir so we don't get errors if the path already exists
  fs::create_dir_all(&app_dir_usr_bin)?;
  fs::create_dir_all(app_dir_usr_lib)?;

  // Copy bins and libs that linuxdeploy doesn't know about

  // we also check if the user may have provided their own copy already
  // xdg-open will be handled by the `files` config instead
  if settings.deep_link_protocols().is_some() && !app_dir_usr_bin.join("xdg-open").exists() {
    fs::copy("/usr/bin/xdg-mime", app_dir_usr_bin.join("xdg-mime"))
      .fs_context("xdg-mime binary not found", "/usr/bin/xdg-mime".to_string())?;
  }

  // we also check if the user may have provided their own copy already
  if settings.appimage().bundle_xdg_open && !app_dir_usr_bin.join("xdg-open").exists() {
    fs::copy("/usr/bin/xdg-open", app_dir_usr_bin.join("xdg-open"))
      .fs_context("xdg-open binary not found", "/usr/bin/xdg-open".to_string())?;
  }

  let search_dirs = [
    match settings.binary_arch() {
      Arch::X86_64 => "/usr/lib/x86_64-linux-gnu/",
      Arch::X86 => "/usr/lib/i386-linux-gnu/",
      Arch::AArch64 => "/usr/lib/aarch64-linux-gnu/",
      Arch::Armhf => "/usr/lib/arm-linux-gnueabihf/",
      _ => unreachable!(),
    },
    "/usr/lib64",
    "/usr/lib",
    "/usr/libexec",
  ];

  for file in [
    "WebKitNetworkProcess",
    "WebKitWebProcess",
    "injected-bundle/libwebkit2gtkinjectedbundle.so",
  ] {
    for source in search_dirs.map(PathBuf::from) {
      // TODO: Check if it's the same dir name on all systems
      let source = source.join("webkit2gtk-4.1").join(file);
      if source.exists() {
        fs_utils::copy_file(
          &source,
          &app_dir_path.join(source.strip_prefix("/").unwrap()),
        )?;
      }
    }
  }

  fs::copy(
    tools_path.join(format!("AppRun-{tools_arch}")),
    app_dir_path.join("AppRun"),
  )?;
  fs::copy(
    app_dir_path.join(larger_icon_path),
    app_dir_path.join(format!("{product_name}.png")),
  )?;
  std::os::unix::fs::symlink(format!("{product_name}.png"), app_dir_path.join(".DirIcon"))?;
  std::os::unix::fs::symlink(
    format!("usr/share/applications/{product_name}.desktop"),
    app_dir_path.join(format!("{product_name}.desktop")),
  )?;

  let log_level = match settings.log_level() {
    log::Level::Error => "3",
    log::Level::Warn => "2",
    log::Level::Info => "1",
    _ => "0",
  };

  let mut cmd = Command::new(linuxdeploy_path);
  cmd.env("OUTPUT", &appimage_path);
  cmd.env("ARCH", tools_arch);
  // Looks like the cli arg isn't enough for the updated AppImage output-plugin.
  cmd.env("APPIMAGE_EXTRACT_AND_RUN", "1");
  cmd.args([
    "--appimage-extract-and-run",
    "--verbosity",
    log_level,
    "--appdir",
    &app_dir_path.display().to_string(),
    "--plugin",
    "gtk",
  ]);
  if settings.appimage().bundle_media_framework {
    cmd.args(["--plugin", "gstreamer"]);
  }
  cmd.args(["--output", "appimage"]);

  // Linuxdeploy logs everything into stderr so we have to ignore the output ourselves here
  if settings.log_level() == log::Level::Error {
    log::debug!(action = "Running"; "Command `linuxdeploy {}`", cmd.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));
    if !cmd.output()?.status.success() {
      return Err(crate::Error::GenericError(
        "failed to run linuxdeploy".to_string(),
      ));
    }
  } else {
    cmd.output_ok()?;
  }

  fs::remove_dir_all(&package_dir)?;
  Ok(vec![appimage_path])
}

fn apprun_sha256(arch: &str) -> crate::Result<&'static str> {
  match arch {
    "x86_64" => Ok("f30140a43a0a59e46db21bdefdf749b9e9f2c6946e92afabbacf98b8ae73fb4f"),
    "i686" => Ok("a573a682b1a4a3e9b5dddbd1f5785749b7bba6013149b51ae99d0f123fe11691"),
    "aarch64" => Ok("072f17c0895a85c490282fe5395c5007e5fc75da727e553b3b8fb680feb11578"),
    "armhf" => Ok("b14d89f0762bcf09fc6af2359d936675928b8833ff52a1aeda68cb28a309f6ba"),
    _ => Err(crate::Error::GenericError(format!(
      "Unsupported AppRun architecture: {arch}"
    ))),
  }
}

fn linuxdeploy_sha256(arch: &str) -> crate::Result<&'static str> {
  match arch {
    "x86_64" => Ok("e762bea85c8eb0d4b3508d46e5c1f037f717d0f9303ae3b4aafc8b04991fa1ef"),
    "i386" => Ok("de64d95d33beb8d9846128689281d9e01403ea274291b3be8dadac1c33d53bc6"),
    "aarch64" => Ok("b12b5cc57bd0921e1f98d73f58aa364503bc1a27f54b7a69fd2870bce7fa2f55"),
    "armhf" => Ok("d533831caec5ec82bf88673c123af42b3776357a972bb3bb673ef71ea7f9e117"),
    _ => Err(crate::Error::GenericError(format!(
      "Unsupported linuxdeploy architecture: {arch}"
    ))),
  }
}

fn appimage_plugin_sha256(arch: &str) -> crate::Result<&'static str> {
  match arch {
    "x86_64" => Ok("1da16a46fa5e058ae740e7c35ed0d36d86cb869ac9cc8a5fd9a1847d7978d99a"),
    "i686" => Ok("cae61f394ecef9823a40a9b574663dca4831dc8933be39c408f95468cb748a90"),
    "aarch64" => Ok("a89f5784a86b6c014898b6bd24c6f4e3cc76727814222099934b72d74df7e4e6"),
    "armhf" => Ok("977d6d3cbdbe95b604e16fe5ecd0a77f83da1fe57eff03a2971614a6f3603c7b"),
    _ => Err(crate::Error::GenericError(format!(
      "Unsupported AppImage plugin architecture: {arch}"
    ))),
  }
}

fn ensure_verified_tool(path: &Path, url: &str, sha256: &str) -> crate::Result<()> {
  if path.exists() {
    if verify_file_hash(path, sha256, HashAlgorithm::Sha256).is_ok() {
      make_executable(path)?;
      return Ok(());
    }

    log::warn!(
      "Cached tool at {} failed integrity verification. Redownloading it.",
      path.display()
    );
    fs::remove_file(path)?;
  }

  let data = download_and_verify(url, sha256, HashAlgorithm::Sha256)?;
  write_and_make_executable(path, data)?;
  Ok(())
}

fn make_executable(path: &Path) -> std::io::Result<()> {
  use std::os::unix::fs::PermissionsExt;

  fs::set_permissions(path, fs::Permissions::from_mode(0o770))
}

fn copy_and_make_executable(source: &Path, destination: &Path) -> crate::Result<()> {
  fs::copy(source, destination)?;
  make_executable(destination)?;
  Ok(())
}

// returns the linuxdeploy path to keep linuxdeploy_arch contained
fn prepare_tools(tools_path: &Path, arch: &str, verbose: bool) -> crate::Result<PathBuf> {
  let apprun = tools_path.join(format!("AppRun-{arch}"));
  ensure_verified_tool(
    &apprun,
    &format!(
      "https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-{arch}"
    ),
    apprun_sha256(arch)?,
  )?;

  let linuxdeploy_arch = if arch == "i686" { "i386" } else { arch };
  let linuxdeploy = tools_path.join(format!("linuxdeploy-{linuxdeploy_arch}.AppImage"));
  ensure_verified_tool(
    &linuxdeploy,
    &format!("https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy/linuxdeploy-{linuxdeploy_arch}.AppImage"),
    linuxdeploy_sha256(linuxdeploy_arch)?,
  )?;

  let gtk = tools_path.join("linuxdeploy-plugin-gtk.sh");
  write_and_make_executable(&gtk, include_bytes!("linuxdeploy-plugin-gtk.sh").to_vec())?;

  let gstreamer = tools_path.join("linuxdeploy-plugin-gstreamer.sh");
  write_and_make_executable(
    &gstreamer,
    include_bytes!("linuxdeploy-plugin-gstreamer.sh").to_vec(),
  )?;

  let appimage = tools_path.join("linuxdeploy-plugin-appimage.AppImage");
  if let Err(err) = ensure_verified_tool(
    &appimage,
    &format!("https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-{arch}.AppImage"),
    appimage_plugin_sha256(arch)?,
  ) {
    log::error!("Download of AppImage plugin failed. Using older built-in version instead.");
    if verbose {
      log::debug!("{err:?}");
    }
    if appimage.exists() {
      if let Err(remove_err) = fs::remove_file(&appimage) {
        log::debug!(
          "Failed to remove unverified AppImage plugin at {}: {remove_err:?}",
          appimage.display()
        );
      }
    }
  }

  let linuxdeploy_run = tools_path.join(format!("linuxdeploy-{linuxdeploy_arch}-run.AppImage"));
  copy_and_make_executable(&linuxdeploy, &linuxdeploy_run)?;

  // This should prevent linuxdeploy to be detected by appimage integration tools
  let _ = Command::new("dd")
    .args([
      "if=/dev/zero",
      "bs=1",
      "count=3",
      "seek=8",
      "conv=notrunc",
      &format!("of={}", linuxdeploy_run.display()),
    ])
    .output();

  Ok(linuxdeploy_run)
}
