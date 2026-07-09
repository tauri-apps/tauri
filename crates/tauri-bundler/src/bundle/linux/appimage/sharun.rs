// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs, path::PathBuf, process::Command};

use anyhow::Context;

use crate::{
  bundle::{linux::debian, settings::Arch},
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

  let package_dir = settings
    .project_out_directory()
    .join("bundle/appimage_deb/");

  let main_binary = settings.main_binary()?;
  let product_name = settings.product_name();

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

  // generate deb_folder structure
  let (data_dir, icons) = debian::generate_data(&settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.appimage().files, &data_dir)
    .with_context(|| "Failed to copy custom files")?;

  fs::create_dir_all(&output_path)?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.product_name()));
  let appimage_filename = format!(
    "{}_{}_{appimage_arch}.AppImage",
    settings.product_name(),
    settings.version_string()
  );
  let appimage_path = output_path.join(&appimage_filename);

  let larger_icon = icons
    .iter()
    .filter(|i| i.width == i.height)
    .max_by_key(|i| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  // TODO:
  let _verbosity = match settings.log_level() {
    log::Level::Error => "-q", // errors only
    log::Level::Info => "",    // errors + "normal logs" (mostly rpath)
    log::Level::Trace => "-v", // You can expect way over 1k lines from just lib4bin on this level
    _ => "",
  };

  let bins = settings.copy_binaries(&app_dir_path.join("usr/bin/"))?;
  let bins = bins
    .iter()
    .map(|b| format!(" \"{}\"", b.to_string_lossy()))
    .collect::<String>();

  let needs_tty = !dbg!(Command::new("/bin/sh").arg("-c").arg("set -m").status())?.success();

  let mut cmd = Command::new("/bin/sh");
  cmd
    .current_dir(&output_path)
    .env("APPDIR", &app_dir_path)
    .env("OUTNAME", &appimage_filename)
    .env(
      "DESKTOP",
      data_dir.join(format!("usr/share/applications/{product_name}.desktop")),
    )
    .env("ICON", &larger_icon.path)
    .env("OUTPUT_APPIMAGE", "1")
    //.env("URUNTIME2APPIMAGE_SOURCE", "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/uruntime2appimage.sh")
    //.env("ADD_HOOKS", "fix-namespaces.hook")
    .args([
      "-c",
      &format!(
        r#"{}"{}" "{}" {bins}{}"#,
        if needs_tty {"script -q -c '"} else {""},
        quick_sharun.to_string_lossy(),
        data_dir
          .join(format!("usr/bin/{}", main_binary.name()))
          .to_string_lossy(),
          if needs_tty {"'"} else {""},
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

  {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&appimage_path, fs::Permissions::from_mode(0o770))?;
  }

  fs::remove_dir_all(package_dir)?;
  Ok(vec![appimage_path])
}
