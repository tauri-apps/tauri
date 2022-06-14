// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    path::{PathBuf, Path},
    collections::BTreeSet, 
    ffi::OsStr, 
    fs::File,
    io::Write,
};

use image::{codecs::png::PngDecoder, ImageDecoder};

use crate::{Settings, bundle::common};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LinuxIcon {
  pub width: u32,
  pub height: u32,
  pub is_high_density: bool,
  pub path: PathBuf,
}

/// Generate the icon files and store them under the `data_dir`.
pub fn generate_icon_files(settings: &Settings, data_dir: &Path) -> crate::Result<BTreeSet<LinuxIcon>> {
    let base_dir = data_dir.join("usr/share/icons/hicolor");
    let get_dest_path = |width: u32, height: u32, is_high_density: bool| {
      base_dir.join(format!(
        "{}x{}{}/apps/{}.png",
        width,
        height,
        if is_high_density { "@2" } else { "" },
        settings.main_binary_name()
      ))
    };
    let mut icons = BTreeSet::new();
    for icon_path in settings.icon_files() {
      let icon_path = icon_path?;
      if icon_path.extension() != Some(OsStr::new("png")) {
        continue;
      }
      // Put file in scope so that it's closed when copying it
      let linux_icon = {
        let decoder = PngDecoder::new(File::open(&icon_path)?)?;
        let width = decoder.dimensions().0;
        let height = decoder.dimensions().1;
        let is_high_density = common::is_retina(&icon_path);
        let dest_path = get_dest_path(width, height, is_high_density);
        LinuxIcon {
          width,
          height,
          is_high_density,
          path: dest_path,
        }
      };
      if !icons.contains(&linux_icon) {
        common::copy_file(&icon_path, &linux_icon.path)?;
        icons.insert(linux_icon);
      }
    }
    Ok(icons)
}


/// Generate the application desktop file and store it under the `data_dir`.
pub fn generate_desktop_file(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
    let bin_name = settings.main_binary_name();
    let desktop_file_name = format!("{}.desktop", bin_name);
    let desktop_file_path = data_dir
      .join("usr/share/applications")
      .join(desktop_file_name);
    let file = &mut common::create_file(&desktop_file_path)?;
    // For more information about the format of this file, see
    // https://developer.gnome.org/integration-guide/stable/desktop-files.html.en
    writeln!(file, "[Desktop Entry]")?;
    if let Some(category) = settings.app_category() {
      writeln!(file, "Categories={}", category.gnome_desktop_categories())?;
    } else {
      writeln!(file, "Categories=")?;
    }
    if !settings.short_description().is_empty() {
      writeln!(file, "Comment={}", settings.short_description())?;
    }
    writeln!(file, "Exec={}", bin_name)?;
    writeln!(file, "Icon={}", bin_name)?;
    writeln!(file, "Name={}", settings.product_name())?;
    writeln!(file, "Terminal=false")?;
    writeln!(file, "Type=Application")?;
    Ok(())
}
