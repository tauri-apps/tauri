use goblin::elf::Elf;
use std::path::PathBuf;

use crate::PackageType;


/// Change value of __TAURI_BUNDLE_TYPE statis variale to mark which package type if was bundled in
pub fn patch_binary(binary_path: &PathBuf, package_type: &PackageType) -> crate::Result<()> {
  let mut file_data = std::fs::read(binary_path).expect("Could not binary read file.");

  let elf = match goblin::Object::parse(&file_data)? {
    goblin::Object::Elf(elf) => elf,
    _ => return Err(crate::Error::BinaryParseError("Not an ELF file".into())),
  };

  if let Some(offset) = find_bundle_type_symbol(elf) {
    let offset = offset as usize;
    if offset + 3 <= file_data.len() {
      let chars = &mut file_data[offset..offset + 3];
      match package_type {
        PackageType::Deb => chars.copy_from_slice(b"DEB"),
        PackageType::Rpm => chars.copy_from_slice(b"RPM"),
        PackageType::AppImage => chars.copy_from_slice(b"APP"),
        _ => {
          return Err(crate::Error::InvalidPackageType(
            package_type.short_name().to_owned(),
            "linux".to_owned(),
          ))
        }
      }
      if let Err(error) = std::fs::write(binary_path, &file_data) {
        return Err(crate::Error::BinaryWriteError(error.to_string()));
      }
    } else {
      return Err(crate::Error::BinaryOffsetOutOfRange);
    }
  } else {
    return Err(crate::Error::MissingBundleTypeVar);
  }

  Ok(())
}

/// Find address of a symbol in relocations table
fn find_bundle_type_symbol(elf: Elf<'_>) -> Option<i64> {
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
