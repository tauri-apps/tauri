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
  bundle::{linux::debian, settings::Arch},
  utils::{fs_utils, http_utils::download, CommandExt},
  Settings,
};

use super::write_and_make_executable;

// TODO: Maybe bundle xdg-open and maybeee xdg-mime as a fallback
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

  let (sharun_aio, uruntime, uruntime_lite) =
    prepare_tools(&tools_path, tools_arch, settings.appimage().squashfs)?;

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

  let upinfo = std::env::var("UPINFO")
    .ok()
    .or(settings.appimage().update_information.clone());

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

  fs_utils::copy_dir(&data_dir, &app_dir_path)?;

  let app_dir_share = &app_dir_path.join("share/");

  fs_utils::copy_dir(&data_dir.join("usr/share/"), app_dir_share)?;

  // The appimage spec allows a symlink but sharun doesn't
  fs::copy(
    app_dir_share.join(format!("applications/{product_name}.desktop")),
    app_dir_path.join(format!("{product_name}.desktop")),
  )?;

  // This could be a symlink as well (supported by sharun as far as i can tell)
  fs::copy(
    app_dir_path.join(larger_icon_path.strip_prefix("usr/").unwrap()),
    app_dir_path.join(format!("{product_name}.png")),
  )?;

  fs::create_dir(app_dir_path.join("bin/"))?;

  // TODO: Test this outside of wsl
  if settings.deep_link_protocols().is_some() {
    write_and_make_executable(
      &app_dir_path.join("bin/xdg-mime"),
      br#"
#!/bin/sh
shift
xdg-mime "$@"
"#
      .to_vec(),
    )?;
  }

  // TODO: Test this outside of wsl
  if settings.appimage().bundle_xdg_open {
    write_and_make_executable(
      &app_dir_path.join("bin/xdg-open"),
      br#"
#!/bin/sh
shift
xdg-open "$@"
"#
      .to_vec(),
    )?;
  }

  std::os::unix::fs::symlink(
    app_dir_path.join(format!("{product_name}.png")),
    app_dir_path.join(".DirIcon"),
  )?;

  let verbosity = match settings.log_level() {
    log::Level::Error => "-q", // errors only
    log::Level::Info => "",    // errors + "normal logs" (mostly rpath)
    log::Level::Trace => "-v", // You can expect way over 1k lines from just lib4bin on this level
    _ => "",
  };

  // TODO: Maybe missing alsa, pipewire, whatever?
  let gst = if settings.appimage().bundle_media_framework {
    format!(
      r#"
/usr/lib/{tools_arch}-linux-gnu/libpulsecommon* \
/usr/lib/{tools_arch}-linux-gnu/gstreamer-1.0/* \
/usr/lib/{tools_arch}-linux-gnu/gstreamer1.0/gstreamer-1.0/* \
"#
    )
  } else {
    "".to_string()
  };

  let bins = settings.copy_binaries(&app_dir_path.join("usr/bin/"))?;
  let bins = bins
    .iter()
    .map(|b| format!(" \"{}\"", b.to_string_lossy()))
    .collect::<String>();

  // TODO: Optional xvfb-run (check if it exists or whatever and then use it)
  // TODO: Check if we can make parts of the opengl (incl. libvulkan) deps optional
  Command::new("/bin/sh")
    .current_dir(&app_dir_path)
    .args([
      "-c",
      &format!(
        r#""{}" l -p {verbosity} -s -k "{}" {} \
/usr/lib/{tools_arch}-linux-gnu/libwebkit2gtk-4.1* \{gst}
/usr/lib/{tools_arch}-linux-gnu/gdk-pixbuf-*/*/*/* \
/usr/lib/{tools_arch}-linux-gnu/gio/modules/* \
/usr/lib/{tools_arch}-linux-gnu/libnss*.so* \
/usr/lib/{tools_arch}-linux-gnu/libGL* \
/usr/lib/{tools_arch}-linux-gnu/libEGL* \
/usr/lib/{tools_arch}-linux-gnu/libvulkan* \
/usr/lib/{tools_arch}-linux-gnu/dri/* \
/usr/lib/{tools_arch}-linux-gnu/gbm/*
"#,
        sharun_aio.to_string_lossy(),
        &app_dir_path
          .join(format!("usr/bin/{}", main_binary.name()))
          .to_string_lossy(),
        bins
      ),
    ])
    .output_ok()
    .context("lib4bin command failed to run.")?;

  fs_utils::remove_dir_all(&app_dir_path.join("usr/"))?;

  let sharun = app_dir_path.join("sharun");
  fs::copy(&sharun, app_dir_path.join("AppRun"))?;

  Command::new(sharun)
    .current_dir(&app_dir_path)
    .arg("-g")
    .output_ok()
    .context("Failed to generate library path for AppDir.")?;

  if let Some(upinfo) = upinfo.as_deref() {
    Command::new(&uruntime_lite)
      .current_dir(&app_dir_path)
      .args([
        "--appimage-addupdinfo",
        &upinfo.replace("$ARCH", tools_arch),
      ])
      .output_ok()
      .context("Failed to add update info.")?;
  }

  // TODO: verbosity - uruntime doesn't expose any settings and doesn't log much
  Command::new(&uruntime)
    .env("ARCH", tools_arch)
    .args([
      "--appimage-mkdwarfs",
      "-f",
      "--set-owner",
      "0",
      "--set-group",
      "0",
      "--no-history",
      "--no-create-timestamp",
      "--compression",
      "zstd:level=22",
      "-S26",
      "-B8",
      "--header",
      &uruntime_lite.to_string_lossy(),
      "-i",
      &app_dir_path.to_string_lossy(),
      "-o",
      &appimage_path.to_string_lossy(),
    ])
    .output_ok()
    .context("Failed to generate AppImage from AppDir.")?;

  {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&appimage_path, fs::Permissions::from_mode(0o770))?;
  }

  if upinfo.is_some() {
    Command::new("zsyncmake")
      .args([
        &appimage_path.to_string_lossy(),
        "-u",
        &appimage_path.to_string_lossy(),
      ])
      .output_ok()
      .context("Failed to create .zsync file.")?;
  }

  fs::remove_dir_all(package_dir)?;
  Ok(vec![appimage_path])
}

// TODO: versions / mirror
fn prepare_tools(
  tools_path: &Path,
  arch: &str,
  squashfs: bool,
) -> crate::Result<(PathBuf, PathBuf, PathBuf)> {
  let fstype = if squashfs { "squashfs" } else { "dwarfs" };
  let uruntime = tools_path.join(format!("uruntime-appimage-{fstype}-{arch}"));
  if !uruntime.exists() {
    let data = download(&format!("https://github.com/VHSgunzo/uruntime/releases/latest/download/uruntime-appimage-{fstype}-{arch}"))?;
    write_and_make_executable(&uruntime, data)?;
  }

  let uruntime_lite = tools_path.join(format!("uruntime-appimage-{fstype}-lite-{arch}"));
  if !uruntime_lite.exists() {
    let data = download(&format!("https://github.com/VHSgunzo/uruntime/releases/latest/download/uruntime-appimage-{fstype}-lite-{arch}"))?;
    write_and_make_executable(&uruntime_lite, data)?;
  }

  // let lib4bin = tools_path.join(format!("lib4bin-{arch}"));
  // if !lib4bin.exists() {
  //   let data =
  //     download("https://raw.githubusercontent.com/VHSgunzo/sharun/refs/heads/main/lib4bin")?;
  //   write_and_make_executable(&lib4bin, data)?;
  // }

  let sharun_aio = tools_path.join(format!("sharun-{arch}-aio"));
  if !sharun_aio.exists() {
    let data = download(&format!(
      "https://github.com/VHSgunzo/sharun/releases/latest/download/sharun-{arch}-aio"
    ))?;
    write_and_make_executable(&sharun_aio, data)?;
  }

  Ok((sharun_aio, uruntime, uruntime_lite))
}
