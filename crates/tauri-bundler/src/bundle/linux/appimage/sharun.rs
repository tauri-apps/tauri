// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Context;

use crate::{
  bundle::{linux::freedesktop, settings::Arch},
  utils::{fs_utils, http_utils::download, CommandExt},
  Settings,
};

use super::write_and_make_executable;

// TODO: Test if bundling xdg-mime makes sense (eg does it even work if it's not on the host system?)
// TODO: Monitor TLS support / certificates - seems to be working in initial tests
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // for backwards compat we keep the amd64 and i386 rewrites in the filename
  let appimage_arch = match settings.binary_arch() {
    Arch::X86_64 => "amd64",
    //Arch::X86 => "i386",
    Arch::AArch64 => "aarch64",
    //Arch::Armhf => "armhf",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }

  let appimage_filename = format!(
    "{}_{}_{appimage_arch}.AppImage",
    settings.product_name(),
    settings.version_string()
  );
  let appimage_path = output_path.join(&appimage_filename);

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  let tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| output_path.to_path_buf(), |p| p.join("tauri"))
    });

  fs::create_dir_all(&tools_path)?;

  // TODO: mirror
  let quick_sharun = tools_path.join("quick-sharun.sh");
  if !quick_sharun.exists() {
    let data = download(
      "https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh",
    )?;
    write_and_make_executable(&quick_sharun, data)?;
  }

  let main_binary = settings.main_binary()?;

  let mut settings = settings.clone();
  if main_binary.name().contains(' ') {
    let main_binary_path = settings.binary_path(main_binary);
    let project_out_dir = settings.project_out_directory();

    let main_binary_name_kebab = heck::AsKebabCase(main_binary.name()).to_string();
    let new_path = project_out_dir.join(&main_binary_name_kebab);
    fs::copy(main_binary_path, new_path)?;

    let main_binary = settings.main_binary_mut()?;
    main_binary.set_name(main_binary_name_kebab);
  }
  let settings = settings;

  fs::create_dir_all(&output_path)?;
  let app_dir = output_path.join(format!("{}.AppDir", settings.product_name()));
  let app_dir_bin = app_dir.join("bin/");
  let app_dir_lib = app_dir.join("lib/");

  // Copy external binaries (externalBin)
  let mut bins = settings
    .copy_binaries(&app_dir_bin)
    .with_context(|| "Failed to copy external binaries")?;

  // Copy Cargo project binaries
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let trgt = app_dir_bin.join(bin.name());
    fs_utils::copy_file(&bin_path, &trgt)
      .with_context(|| format!("Failed to copy binary from {bin_path:?} to {trgt:?}"))?;
    bins.push(trgt);
  }

  settings
    .copy_resources(&app_dir_lib.join(settings.product_name()))
    .with_context(|| "Failed to copy resource files")?;

  let desktop_file = freedesktop::generate_desktop_file(&settings, &None, &app_dir)
    .with_context(|| "Failed to create desktop file")?
    .0;
  fs::rename(
    desktop_file,
    app_dir.join(format!("{}.desktop", settings.product_name())),
  )
  .with_context(|| "Failed to move desktop file")?;

  fs_utils::copy_custom_files(&settings.appimage().files, &app_dir)
    .with_context(|| "Failed to copy custom files")?;

  let icons = freedesktop::list_icon_files(&settings, Path::new(""))
    .with_context(|| "Failed to create icon files")?;

  let largest_icon = icons
    .into_iter()
    .filter(|(i, _)| i.width == i.height)
    .max_by_key(|(i, _)| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");

  let bins = bins
    .iter()
    .map(|p| format!(" \"{}\"", p.to_string_lossy()))
    .collect::<String>();

  let mut cmd = Command::new("/bin/sh");
  cmd
    .current_dir(&output_path)
    .env("APPDIR", &app_dir)
    .env("OUTNAME", &appimage_filename)
    .env("ICON", largest_icon.0.path)
    .env("OUTPUT_APPIMAGE", "1")
    //.env("URUNTIME2APPIMAGE_SOURCE", "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/uruntime2appimage.sh")
    //.env("ADD_HOOKS", "fix-namespaces.hook")
    .args([
      "-c",
      &format!(
        r#""{}" {bins} "{}""#,
        quick_sharun.to_string_lossy(),
        app_dir_lib.to_string_lossy()
      ),
    ]);

  if let Some(upinfo) = std::env::var("UPINFO")
    .ok()
    .or(settings.appimage().update_information.clone())
  {
    cmd.env("UPINFO", upinfo);
  }

  cmd
    .output_ok()
    .context("quick-sharun command failed to run.")?;

  Ok(vec![appimage_path])
}
