// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::debian;
use crate::{
  bundle::settings::Arch,
  utils::{fs_utils, http_utils::download, CommandExt},
  Settings,
};
use anyhow::Context;
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
        "Unsupported architecture: {:?}",
        target
      )));
    }
  };

  let tools_arch = settings.target().split('-').next().unwrap();
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

  let linuxdeploy_path = prepare_tools(&tools_path, tools_arch)?;

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
      .context("xdg-mime binary not found")?;
  }

  // we also check if the user may have provided their own copy already
  if settings.appimage().bundle_xdg_open && !app_dir_usr_bin.join("xdg-open").exists() {
    fs::copy("/usr/bin/xdg-open", app_dir_usr_bin.join("xdg-open"))
      .context("xdg-open binary not found")?;
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
  std::os::unix::fs::symlink(
    app_dir_path.join(format!("{product_name}.png")),
    app_dir_path.join(".DirIcon"),
  )?;
  std::os::unix::fs::symlink(
    app_dir_path.join(format!("usr/share/applications/{product_name}.desktop")),
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

// returns the linuxdeploy path to keep linuxdeploy_arch contained
fn prepare_tools(tools_path: &Path, arch: &str) -> crate::Result<PathBuf> {
  let apprun = tools_path.join(format!("AppRun-{arch}"));
  if !apprun.exists() {
    let data = download(&format!(
      "https://github.com/AppImage/AppImageKit/releases/download/continuous/AppRun-{arch}"
    ))?;
    write_and_make_executable(&apprun, data)?;
  }

  let linuxdeploy_arch = if arch == "i686" { "i383" } else { arch };
  let linuxdeploy = tools_path.join(format!("linuxdeploy-{linuxdeploy_arch}.AppImage"));
  if !linuxdeploy.exists() {
    let data = download(&format!("https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy/linuxdeploy-{linuxdeploy_arch}.AppImage"))?;
    write_and_make_executable(&linuxdeploy, data)?;
  }

  let gtk = tools_path.join("linuxdeploy-plugin-gtk.sh");
  if !gtk.exists() {
    let data = download("https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh")?;
    write_and_make_executable(&gtk, data)?;
  }

  let gstreamer = tools_path.join("linuxdeploy-plugin-gstreamer.sh");
  if !gstreamer.exists() {
    let data = download("https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gstreamer/master/linuxdeploy-plugin-gstreamer.sh")?;
    write_and_make_executable(&gstreamer, data)?;
  }

  // This should prevent linuxdeploy to be detected by appimage integration tools
  let _ = Command::new("dd")
    .args([
      "if=/dev/zero",
      "bs=1",
      "count=3",
      "seek=8",
      "conv=notrunc",
      &format!("of={}", linuxdeploy.display()),
    ])
    .output();

  Ok(linuxdeploy)
}

fn write_and_make_executable(path: &Path, data: Vec<u8>) -> std::io::Result<()> {
  use std::os::unix::fs::PermissionsExt;

  fs::write(path, data)?;
  fs::set_permissions(path, fs::Permissions::from_mode(0o770))?;

  Ok(())
}
