// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Change value of __TAURI_BUNDLE_TYPE static variable to mark which package type it was bundled in
#[cfg(target_os = "linux")]
pub fn patch_binary(
  binary_path: &std::path::PathBuf,
  package_type: &crate::PackageType,
) -> crate::Result<()> {
  let mut file_data = std::fs::read(binary_path).expect("Could not read binary file.");

  let elf = match goblin::Object::parse(&file_data)? {
    goblin::Object::Elf(elf) => elf,
    _ => return Err(crate::Error::GenericError("Not an ELF file".to_owned())),
  };

  let offset = find_bundle_type_symbol(elf).ok_or(crate::Error::MissingBundleTypeVar)?;
  let offset = offset as usize;
  if offset + 3 <= file_data.len() {
    let chars = &mut file_data[offset..offset + 3];
    match package_type {
      crate::PackageType::Deb => chars.copy_from_slice(b"DEB"),
      crate::PackageType::Rpm => chars.copy_from_slice(b"RPM"),
      crate::PackageType::AppImage => chars.copy_from_slice(b"APP"),
      _ => {
        return Err(crate::Error::InvalidPackageType(
          package_type.short_name().to_owned(),
          "linux".to_owned(),
        ))
      }
    }

    std::fs::write(binary_path, &file_data)
      .map_err(|error| crate::Error::BinaryWriteError(error.to_string()))?;
  } else {
    return Err(crate::Error::BinaryOffsetOutOfRange);
  }

  Ok(())
}

/// Find address of a symbol in relocations table
#[cfg(target_os = "linux")]
fn find_bundle_type_symbol(elf: goblin::elf::Elf<'_>) -> Option<i64> {
  for sym in elf.syms.iter() {
    if let Some(name) = elf.strtab.get_at(sym.st_name) {
      if name == "__TAURI_BUNDLE_TYPE" {
        for reloc in elf.dynrelas.iter() {
          if reloc.r_offset == sym.st_value {
            return Some(reloc.r_addend.unwrap());
          }
        }
      }
    }
  }

  None
}
