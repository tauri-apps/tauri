// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Context;
use walkdir::WalkDir;

use crate::{
  Settings,
  bundle::{linux::freedesktop, settings::Arch},
  utils::{CommandExt, fs_utils, http_utils::download},
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
  //let tools_arch = settings.target().split('-').next().unwrap();

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }

  let product_name = settings.product_name();

  let appimage_filename = format!(
    "{}_{}_{appimage_arch}.AppImage",
    product_name,
    settings.version_string()
  );
  let appimage_path = output_path.join(&appimage_filename);

  let tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| output_path.to_path_buf(), |p| p.join("tauri"))
    });

  fs::create_dir_all(&tools_path)?;

  let quick_sharun = tools_path.join("quick-sharun.sh");
  // TODO: offline build support
  // github doesn't send a Last-Modified header
  // if !quick_sharun.exists() {}
  let data = download(
    "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh",
  )?;
  write_and_make_executable(&quick_sharun, data)?;

  // This should come after the download or users will think it's stuck on the download step.
  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  let mut settings = settings.clone();
  if settings.main_binary()?.name().contains(' ') {
    let main_binary = settings.main_binary()?;

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
  let app_dir = output_path.join(format!("{product_name}.AppDir"));
  let app_dir_bin = app_dir.join("bin/");
  let app_dir_lib = app_dir.join("lib/");

  let desktop_file = freedesktop::generate_desktop_file(&settings, &None, &app_dir)
    .with_context(|| "Failed to create desktop file")?
    .0;
  fs::rename(
    desktop_file,
    app_dir.join(format!("{product_name}.desktop")),
  )
  .with_context(|| "Failed to move desktop file")?;
  let _ = fs_utils::remove_dir_all(&app_dir.join("usr/"));

  // Copy Cargo project binaries
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let trgt = app_dir_bin.join(bin.name());
    fs_utils::copy_file(&bin_path, &trgt)
      .with_context(|| format!("Failed to copy binary from {bin_path:?} to {trgt:?}"))?;
  }

  // Copy external binaries (externalBin)
  settings
    .copy_binaries(&app_dir_bin)
    .with_context(|| "Failed to copy external binaries")?;

  settings
    .copy_resources(&app_dir_lib.join(product_name))
    .with_context(|| "Failed to copy resource files")?;

  fs_utils::copy_custom_files(&settings.appimage().files, &app_dir)
    .with_context(|| "Failed to copy custom files")?;

  let icons = freedesktop::list_icon_files(&settings, Path::new(""))
    .with_context(|| "Failed to create icon files")?;

  let largest_icon = icons
    .into_iter()
    .filter(|(i, _)| i.width == i.height)
    .max_by_key(|(i, _)| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");

  fs::copy(largest_icon.1, app_dir.join(format!("{product_name}.png")))
    .with_context(|| "Failed to copy icon file")?;

  fs::create_dir_all(app_dir_bin.join("locales/"))?;

  let cef_path = settings
    .bundle_settings()
    .cef_path
    .clone()
    .expect("this module is only called when cef_path is set");

  let cef_files = [
    // required
    "libcef.so",
    "icudtl.dat",
    "v8_context_snapshot.bin",
    // required end
    // "optional" - but not really since we want support for all of this
    "chrome_100_percent.pak",
    "chrome_200_percent.pak",
    "resources.pak",
    // ANGLE support
    "libEGL.so",
    "libGLESv2.so",
    // SwANGLE support
    "libvk_swiftshader.so",
    "vk_swiftshader_icd.json",
    "libvulkan.so.1",
    // sandbox - may need to be behind a setting?
    "chrome-sandbox",
    // TODO: seccomp
  ];

  for f in cef_files {
    let dest = app_dir_bin.join(f);
    fs::copy(cef_path.join(f), &dest)
      .with_context(|| format!("Failed to copy cef file {f} to {}", dest.display()))?;
    // quick-sharun checks for the NO_STRIP env but libcef.so is 1.5GB so we make sure it's stripped anyway.
    let _ = Command::new("strip").arg(&dest).output_ok();
  }
  let locales = [
    "en-US.pak",
    "en-US_FEMININE.pak",
    "en-US_MASCULINE.pak",
    "en-US_NEUTER.pak",
  ];

  for f in locales {
    fs::copy(
      cef_path.join("locales").join(f),
      app_dir_bin.join("locales").join(f),
    )
    .with_context(|| format!("Failed to copy cef locales file {f}"))?;
  }

  // We need to give quick-sharun the list of binaries AND libraries to include.
  // To support weird `appimage.files` settings we just walk through the whole AppDir we set up.
  // TODO: In some cases we may have to give quick-sharun the path to some directories as well.
  let mut elfs = Vec::new();
  for entry in WalkDir::new(&app_dir) {
    if let Ok(entry) = entry
      && entry.file_type().is_file()
      && is_elf(entry.path())
    {
      elfs.push(entry.path().to_string_lossy().to_string());
    }
  }
  // This is mostly for libappindicator that we added to /usr/lib in tauri-cli/src/interface/rust.rs
  for (target, source) in &settings.appimage().files {
    if target.starts_with("/usr/lib") {
      elfs.push(source.to_string_lossy().to_string());
    }
  }
  let elfs = elfs
    .into_iter()
    .map(|entry| format!(" \"{entry}\""))
    .collect::<String>();

  // TODO: Consider to not rely on quick-sharun when we have more time
  Command::new("/bin/sh")
    .current_dir(&output_path)
    .env("APPDIR", &app_dir)
    // At least on my local machine this was required, worked fine without in CI / using published tauri-apps/cli-cef.
    .env("MAIN_BIN", app_dir_bin.join(settings.main_binary()?.name()))
    .env("OUTPUT_APPIMAGE", "1")
    .env("OUTNAME", &appimage_filename)
    .env("HOOKSRC", "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/hooks")
    .env("DEPLOY_CHROMIUM", "1")
    .env("ADD_HOOKS", "fix-namespaces.hook")
    .args([
      "-c",
      &format!(
        r#""{}" {elfs}"#,
        quick_sharun.to_string_lossy()
      ),
    ])
    .output_ok()
    .context("quick-sharun command failed to run.")?;

  Ok(vec![appimage_path])
}

fn is_elf(path: &Path) -> bool {
  let mut buf = [0; 4];
  if let Ok(mut file) = fs::File::open(path)
    && file.read_exact(&mut buf).is_ok()
  {
    return buf == [0x7f, b'E', b'L', b'F'];
  }
  false
}
