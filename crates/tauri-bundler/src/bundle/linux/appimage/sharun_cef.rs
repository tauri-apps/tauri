// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs, path::PathBuf, process::Command};

use anyhow::Context;

use crate::{
  Settings,
  bundle::{linux::debian, settings::Arch},
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
    "https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh",
  )?;
  write_and_make_executable(&quick_sharun, data)?;

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

  fs::create_dir_all(data_dir.join("usr/bin/"))?;
  fs::create_dir_all(data_dir.join("usr/lib/"))?;
  fs::create_dir_all(data_dir.join("usr/lib/locales"))?;

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
    // ANGEL support
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
    let dest = if f == "chrome-sandbox" {
      data_dir.join("usr/bin/").join(f)
    } else {
      data_dir.join("usr/lib/").join(f)
    };
    fs::copy(cef_path.join(f), &dest)?;
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
      data_dir.join("usr/lib/locales").join(f),
    )?;
  }

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

  // quick-sharun checks the main binary with ldd so even though we manually add the cef files,
  // we'll add them to LD_LIBRARY_PATH to pass the pre-bundle checks
  let mut ld_lib_path = data_dir.join("usr/lib/").to_string_lossy().to_string();
  if let Ok(ld_env) = std::env::var("LD_LIBRARY_PATH") {
    ld_lib_path = format!("{}:{}", ld_lib_path, ld_env);
  }

  let main_binary_path = data_dir.join(format!("usr/bin/{}", main_binary.name()));

  // sharun runs every deployed binary through the bundled dynamic loader
  // (`ld.so --library-path … <binary>`), which only works for position-
  // independent (ET_DYN) executables — an explicit loader invocation on a
  // non-PIE ET_EXEC fails at startup with `object file has no dynamic section`.
  // A Rust `tauri build` binary is already PIE, but a bindings app's main
  // binary is the language runtime (a Node.js Single Executable Application is
  // a copy of the stock, non-PIE `node`), so mark it ET_DYN. Node links at a
  // fixed load address the loader honors, so it still runs correctly; this only
  // affects the copy staged into the AppImage.
  make_position_independent(&main_binary_path)?;

  // The environment quick-sharun reads, shared by the deploy and pack steps.
  let desktop = data_dir.join(format!("usr/share/applications/{product_name}.desktop"));
  let configure = |cmd: &mut Command| {
    cmd
      .current_dir(&output_path)
      .env("APPDIR", &app_dir_path)
      .env("OUTNAME", &appimage_filename)
      .env("DESKTOP", &desktop)
      .env("ICON", &larger_icon.path)
      .env("HOOKSRC", "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/hooks")
      .env("DEPLOY_CHROMIUM", "1")
      .env("ADD_HOOKS", "fix-namespaces.hook")
      // Don't let sharun strip the main binary: a Node SEA carries its payload
      // (assets, config, ACL) in an appended blob that `postject` injected by
      // rewriting the ELF, and `strip` shifts sections around it, corrupting the
      // binary so it no longer loads (`object file has no dynamic section`).
      // Libraries are still stripped (libcef is stripped as it is staged anyway).
      .env("NO_STRIP", "binaries")
      .env("LD_LIBRARY_PATH", &ld_lib_path);
  };

  // Deploy the AppDir but don't pack the AppImage yet (OUTPUT_APPIMAGE unset):
  // sharun replaces every deployed executable with a wrapper stub (a hardlink to
  // its launcher), including the `tauri-cef-helper` that CEF re-executes to spawn
  // its renderer/GPU subprocesses. Launched through that stub those subprocesses
  // never come up and the window stays blank, so we restore the real helper
  // before packing (below).
  // TODO: Consider to not rely on quick-sharun when we have more time
  let mut deploy = Command::new("/bin/sh");
  configure(&mut deploy);
  deploy
    .args([
      "-c",
      &format!(
        r#""{}" "{}" {bins} "{}" "{}""#,
        quick_sharun.to_string_lossy(),
        main_binary_path.to_string_lossy(),
        // TODO: This may have to be in lib instead
        data_dir.join("usr/bin/chrome-sandbox").to_string_lossy(),
        data_dir.join("usr/lib/").to_string_lossy()
      ),
    ])
    .output_ok()
    .context("quick-sharun deploy command failed to run.")?;

  // Restore the real CEF subprocess helper over sharun's stub in the resource
  // dir. The stub is a hardlink to the single sharun binary, so unlink it first —
  // writing onto the stub would corrupt sharun itself — then copy the pristine
  // helper, which resolves its sibling `libcef` in the resource dir through its
  // `$ORIGIN` RUNPATH. Only a bindings app ships this helper (`cef_path` is its
  // staged resource dir); a Rust cef app has none, so this is a no-op there.
  let helper_src = cef_path.join("tauri-cef-helper");
  if helper_src.is_file() {
    let helper_dest = app_dir_path
      .join("lib")
      .join(product_name)
      .join("tauri-cef-helper");
    if helper_dest.exists() {
      use std::os::unix::fs::PermissionsExt;
      fs::remove_file(&helper_dest)
        .with_context(|| format!("failed to remove helper stub {}", helper_dest.display()))?;
      fs::copy(&helper_src, &helper_dest)
        .with_context(|| format!("failed to restore CEF helper to {}", helper_dest.display()))?;
      fs::set_permissions(&helper_dest, fs::Permissions::from_mode(0o755))?;
    } else {
      log::warn!(
        "expected a CEF subprocess helper at {}, but none was found; the packaged app may show a blank window",
        helper_dest.display()
      );
    }
  }

  // Now pack the fixed AppDir into the AppImage.
  let mut pack = Command::new("/bin/sh");
  configure(&mut pack);
  pack
    .args([
      "-c",
      &format!(r#""{}" --make-appimage"#, quick_sharun.to_string_lossy()),
    ])
    .output_ok()
    .context("quick-sharun --make-appimage command failed to run.")?;

  {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&appimage_path, fs::Permissions::from_mode(0o770)).expect("perms");
  }

  fs::remove_dir_all(package_dir).expect("rmdir");
  Ok(vec![appimage_path])
}

/// Marks an ELF executable as position-independent (`ET_DYN`) by patching its
/// header `e_type`, so the AppImage's bundled dynamic loader can launch it (see
/// the call site). A non-PIE (`ET_EXEC`) executable — such as the stock `node`
/// a Node.js Single Executable Application is built from — fails an explicit
/// loader invocation with `object file has no dynamic section`; flipping the
/// type makes it loadable, and it still runs correctly because such binaries
/// link at a fixed address the loader honors.
///
/// A no-op when the file is not an ELF executable or is already `ET_DYN` (a Rust
/// `tauri build` binary, or a PIE bindings runtime), so it is safe to call
/// unconditionally on whatever main binary is being bundled.
fn make_position_independent(path: &std::path::Path) -> crate::Result<()> {
  use std::io::{Read, Seek, SeekFrom, Write};

  let mut file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .with_context(|| {
      format!(
        "failed to open {} to mark it position-independent",
        path.display()
      )
    })?;

  // e_ident (magic + EI_CLASS/EI_DATA …) followed by the 16-bit e_type field.
  let mut header = [0u8; 18];
  if file.read_exact(&mut header).is_err() || &header[0..4] != b"\x7fELF" {
    // Too small to be an ELF, or not one at all — nothing to patch.
    return Ok(());
  }

  // ET_EXEC = 2, ET_DYN = 3. e_type is at offset 0x10, byte order given by
  // EI_DATA (e_ident[5]): 1 = little-endian, 2 = big-endian. Both values fit in
  // the low byte with a zero high byte, so patching the least-significant byte
  // to 3 is enough for either endianness.
  const ET_EXEC: u8 = 2;
  const ET_DYN: u8 = 3;
  let lsb_offset: u64 = if header[5] == 2 { 17 } else { 16 };
  if header[lsb_offset as usize] != ET_EXEC {
    // Already ET_DYN (PIE) or some other type — leave it untouched.
    return Ok(());
  }

  file.seek(SeekFrom::Start(lsb_offset))?;
  file.write_all(&[ET_DYN])?;
  log::debug!("marked {} as position-independent (ET_DYN)", path.display());
  Ok(())
}
