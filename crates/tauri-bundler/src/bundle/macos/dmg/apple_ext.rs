// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Apple's ISO 9660 extensions for the volume root.
//!
//! `genisoimage -apple` writes an `AA` system use entry (file type, creator and
//! Finder flags) for regular files only, so the root directory never carries
//! the "has custom icon" Finder flag. Without that flag Finder ignores
//! `.VolumeIcon.icns` and the mounted image keeps the generic disk icon.
//!
//! The flag lives in the root directory's own (".") record, so this module
//! rewrites that record in the finished ISO before it is converted to UDIF.

use std::{fs, path::Path};

const SECTOR_SIZE: usize = 2048;
/// The primary volume descriptor is the first descriptor after the system area.
const PVD_SECTOR: usize = 16;
/// Offset of the root directory record inside the primary volume descriptor.
const PVD_ROOT_RECORD: usize = 156;
/// `kHasCustomIcon`: Finder reads `.VolumeIcon.icns` for this directory.
const FINDER_FLAG_HAS_CUSTOM_ICON: u16 = 0x0400;
/// `AA` entry: signature, length, version, type, creator, Finder flags.
const AA_ENTRY_LEN: usize = 14;

/// Set the custom icon flag on the volume root of an ISO 9660 image.
pub fn set_volume_custom_icon(iso: &Path) -> crate::Result<()> {
  let mut image = fs::read(iso).map_err(|error| {
    crate::Error::GenericError(format!("failed to read {}: {error}", iso.display()))
  })?;

  let pvd = PVD_SECTOR * SECTOR_SIZE;
  if image.len() < pvd + SECTOR_SIZE || image[pvd + 1..pvd + 6] != *b"CD001" {
    return Err(crate::Error::GenericError(format!(
      "{} is not an ISO 9660 image",
      iso.display()
    )));
  }

  let root = &image[pvd + PVD_ROOT_RECORD..pvd + PVD_ROOT_RECORD + 34];
  let extent = u32::from_le_bytes(root[2..6].try_into().expect("root extent")) as usize;
  let size = u32::from_le_bytes(root[10..14].try_into().expect("root size")) as usize;
  let start = extent * SECTOR_SIZE;
  let end = start + size;
  if end > image.len() {
    return Err(crate::Error::GenericError(format!(
      "root directory of {} points outside the image",
      iso.display()
    )));
  }

  let mut records = read_directory(&image[start..end])?;
  let root_index = records
    .iter()
    .position(|record| record.len() >= 34 && record[32] == 1 && record[33] == 0)
    .ok_or_else(|| {
      crate::Error::GenericError(format!("{} has no root directory record", iso.display()))
    })?;
  set_custom_icon_flag(&mut records[root_index])?;

  let written = write_directory(&records, &mut image[start..end]);
  if written > size {
    return Err(crate::Error::GenericError(format!(
      "root directory of {} does not have room for the icon entry",
      iso.display()
    )));
  }

  fs::write(iso, &image).map_err(|error| {
    crate::Error::GenericError(format!("failed to write {}: {error}", iso.display()))
  })?;
  Ok(())
}

/// Split a directory extent into its records, dropping the sector padding.
fn read_directory(extent: &[u8]) -> crate::Result<Vec<Vec<u8>>> {
  let mut records = Vec::new();
  let mut offset = 0;
  while offset < extent.len() {
    if extent[offset] == 0 {
      // Records never cross a sector boundary, so the rest is padding.
      offset = (offset / SECTOR_SIZE + 1) * SECTOR_SIZE;
      continue;
    }

    let length = extent[offset] as usize;
    let end = offset
      .checked_add(length)
      .ok_or_else(|| crate::Error::GenericError("directory record length overflows".into()))?;
    if length < 34 || end > extent.len() {
      return Err(crate::Error::GenericError(format!(
        "invalid directory record at offset {offset}"
      )));
    }
    records.push(extent[offset..end].to_vec());
    offset = end;
  }
  Ok(records)
}

/// Lay the records back out, padding to the next sector when one does not fit.
fn write_directory(records: &[Vec<u8>], extent: &mut [u8]) -> usize {
  extent.fill(0);
  let mut offset = 0;
  for record in records {
    if offset / SECTOR_SIZE != (offset + record.len() - 1) / SECTOR_SIZE {
      offset = (offset / SECTOR_SIZE + 1) * SECTOR_SIZE;
    }
    if offset + record.len() > extent.len() {
      return usize::MAX;
    }
    extent[offset..offset + record.len()].copy_from_slice(record);
    offset += record.len();
  }
  offset
}

/// Add or update the `AA` entry that carries the Finder flags.
fn set_custom_icon_flag(record: &mut Vec<u8>) -> crate::Result<()> {
  let name_len = record[32] as usize;
  let system_use = 33 + name_len + usize::from(name_len.is_multiple_of(2));

  let mut offset = system_use;
  while offset + 4 <= record.len() {
    let length = record[offset + 2] as usize;
    if length < 4 || offset + length > record.len() {
      break;
    }
    if &record[offset..offset + 2] == b"AA" {
      if length < AA_ENTRY_LEN {
        return Err(crate::Error::GenericError(
          "Apple extension entry is too short".into(),
        ));
      }
      record[offset + 12..offset + 14].copy_from_slice(&FINDER_FLAG_HAS_CUSTOM_ICON.to_be_bytes());
      return Ok(());
    }
    offset += length;
  }

  let mut entry = [0u8; AA_ENTRY_LEN];
  entry[0..2].copy_from_slice(b"AA");
  entry[2] = AA_ENTRY_LEN as u8;
  entry[3] = 2;
  entry[4..8].copy_from_slice(b"TEXT");
  entry[8..12].copy_from_slice(b"unix");
  entry[12..14].copy_from_slice(&FINDER_FLAG_HAS_CUSTOM_ICON.to_be_bytes());
  record.extend_from_slice(&entry);

  let length = u8::try_from(record.len())
    .map_err(|_| crate::Error::GenericError("root directory record is too large".into()))?;
  record[0] = length;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Directory record with a one-byte name and the given system use area.
  fn record(name: u8, system_use: &[u8]) -> Vec<u8> {
    // 33-byte fixed part, one name byte at offset 33, system use from 34.
    let mut record = vec![0u8; 34];
    record[32] = 1;
    record[33] = name;
    record.extend_from_slice(system_use);
    record[0] = record.len() as u8;
    record
  }

  fn aa_entry(flags: u16) -> Vec<u8> {
    let mut entry = vec![0u8; AA_ENTRY_LEN];
    entry[0..2].copy_from_slice(b"AA");
    entry[2] = AA_ENTRY_LEN as u8;
    entry[3] = 2;
    entry[4..8].copy_from_slice(b"TEXT");
    entry[8..12].copy_from_slice(b"unix");
    entry[12..14].copy_from_slice(&flags.to_be_bytes());
    entry
  }

  #[test]
  fn appends_the_entry_when_missing() {
    let mut root = record(0, b"SP\x07\x01\xbe\xef\x00");
    let original = root.len();
    set_custom_icon_flag(&mut root).unwrap();
    assert_eq!(root.len(), original + AA_ENTRY_LEN);
    assert_eq!(root[0] as usize, root.len());
    assert_eq!(&root[original..original + 14], aa_entry(0x0400).as_slice());
  }

  #[test]
  fn updates_an_existing_entry() {
    let mut root = record(0, &aa_entry(0));
    let original = root.len();
    set_custom_icon_flag(&mut root).unwrap();
    assert_eq!(root.len(), original);
    assert_eq!(&root[original - 2..], &[0x04, 0x00]);
  }

  #[test]
  fn round_trips_a_directory_extent() {
    let records = vec![
      record(0, b"SP\x07\x01\xbe\xef\x00"),
      record(1, b""),
      record(b'a', b"AA\x0e\x02TEXTunix\x00\x00"),
    ];
    let mut extent = vec![0u8; SECTOR_SIZE];
    let written = write_directory(&records, &mut extent);
    assert!(written < SECTOR_SIZE);
    let parsed = read_directory(&extent).unwrap();
    assert_eq!(parsed, records);
  }
}
