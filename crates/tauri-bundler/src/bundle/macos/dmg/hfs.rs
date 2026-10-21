// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Post-processing for HFS+ images built without macOS.
//!
//! `kHasCustomIcon` (the Finder flag that makes Finder use `.VolumeIcon.icns`)
//! lives in the root folder's catalog record on an HFS+ volume. There is no
//! tool on Linux that sets it, so the catalog B-tree is patched in place.
//!
//! Only the volume root is touched, which means one leaf record and two bytes.
//! The image is already finalized, so nothing else has to be rebalanced.

use std::{fs, path::Path};

const VOLUME_HEADER_OFFSET: usize = 1024;
const VOLUME_HEADER_LEN: usize = 512;
/// Offset of the catalog fork descriptor inside the volume header.
const CATALOG_FORK: usize = 272;
/// Offset of the volume's 32-byte `finderInfo` inside the volume header.
const FINDER_INFO: usize = 80;
/// The root folder is always catalog node id 2.
const ROOT_FOLDER_ID: u32 = 2;
/// `kHasCustomIcon`.
const FINDER_FLAG_HAS_CUSTOM_ICON: u16 = 0x0400;
const FOLDER_RECORD: u16 = 1;
/// `finderInfo` byte offset inside a folder record: type/creator, flags, ...
const FOLDER_FINDER_FLAGS: usize = 56;

/// Make Finder open the volume, and optionally use its custom icon.
pub fn finalize(image: &Path, has_custom_icon: bool) -> crate::Result<()> {
  let mut bytes = fs::read(image).map_err(|error| {
    crate::Error::GenericError(format!("failed to read {}: {error}", image.display()))
  })?;

  let block_size;
  let extents;
  {
    let header = volume_header(&bytes, image)?;
    block_size = read_u32(&header[40..44]) as usize;
    extents = Fork::extents(header, CATALOG_FORK);
  }
  if extents.is_empty() {
    return Err(crate::Error::GenericError(format!(
      "{} has no catalog fork",
      image.display()
    )));
  }

  // The B-tree is addressed inside the fork, so flatten it first. Extents are
  // block aligned, which keeps every node inside a single extent.
  let mut catalog = Vec::new();
  for (start, count) in &extents {
    let start = *start as usize * block_size;
    let end = start + *count as usize * block_size;
    catalog.extend_from_slice(bytes.get(start..end).ok_or_else(|| {
      crate::Error::GenericError(format!(
        "catalog fork of {} points outside the image",
        image.display()
      ))
    })?);
  }

  if has_custom_icon {
    let flag_offset = find_root_folder_flags(&catalog)?;
    let flags = read_u16(&catalog[flag_offset..flag_offset + 2]) | FINDER_FLAG_HAS_CUSTOM_ICON;
    catalog[flag_offset..flag_offset + 2].copy_from_slice(&flags.to_be_bytes());
  }

  let mut catalog_offset = 0;
  for (start, count) in &extents {
    let start = *start as usize * block_size;
    let end = start + *count as usize * block_size;
    bytes[start..end].copy_from_slice(&catalog[catalog_offset..catalog_offset + (end - start)]);
    catalog_offset += end - start;
  }

  // `bless --openfolder` sets the second `finderInfo` word to the folder to
  // open, which is what makes the window pop up on mount.
  let finder = VOLUME_HEADER_OFFSET + FINDER_INFO + 8;
  bytes[finder..finder + 4].copy_from_slice(&ROOT_FOLDER_ID.to_be_bytes());

  fs::write(image, &bytes).map_err(|error| {
    crate::Error::GenericError(format!("failed to write {}: {error}", image.display()))
  })?;
  Ok(())
}

fn volume_header<'a>(bytes: &'a [u8], image: &Path) -> crate::Result<&'a [u8]> {
  let header = bytes
    .get(VOLUME_HEADER_OFFSET..VOLUME_HEADER_OFFSET + VOLUME_HEADER_LEN)
    .ok_or_else(|| {
      crate::Error::GenericError(format!(
        "{} is too small to be an HFS+ image",
        image.display()
      ))
    })?;
  if &header[0..2] != b"H+" && &header[0..2] != b"HX" {
    return Err(crate::Error::GenericError(format!(
      "{} is not an HFS+ image",
      image.display()
    )));
  }
  Ok(header)
}

/// An HFS+ fork: eight extents addressed in allocation blocks.
struct Fork;

impl Fork {
  fn extents(header: &[u8], offset: usize) -> Vec<(u32, u32)> {
    // logical size, clump size, total blocks, then eight (start, count) pairs.
    (0..8)
      .map(|index| {
        let base = offset + 16 + index * 8;
        (
          read_u32(&header[base..base + 4]),
          read_u32(&header[base + 4..base + 8]),
        )
      })
      .filter(|(_, count)| *count != 0)
      .collect()
  }
}

/// Return the byte offset of the root folder's Finder flags in a catalog file.
fn find_root_folder_flags(catalog: &[u8]) -> crate::Result<usize> {
  let node_size = read_u16(&catalog[32..34]) as usize;
  if node_size == 0 {
    return Err(crate::Error::GenericError(
      "HFS+ catalog has a zero node size".into(),
    ));
  }

  let mut node = read_u32(&catalog[24..28]) as usize;
  while node != 0 {
    let base = node
      .checked_mul(node_size)
      .ok_or_else(|| crate::Error::GenericError("catalog node offset overflows".into()))?;
    let header = catalog
      .get(base..base + 14)
      .ok_or_else(|| crate::Error::GenericError("catalog node points outside the fork".into()))?;
    let next = read_u32(&header[0..4]) as usize;
    let kind = header[8];
    let records = read_u16(&header[10..12]) as usize;
    // 0xff is a leaf node; internal nodes do not hold catalog records.
    if kind != 0xff {
      return Err(crate::Error::GenericError(
        "HFS+ catalog starts at a non-leaf node".into(),
      ));
    }

    for index in 0..records {
      let offset_at = base + node_size - (index + 1) * 2;
      let record = base + read_u16(&catalog[offset_at..offset_at + 2]) as usize;
      let key_len = read_u16(&catalog[record..record + 2]) as usize;
      // The key is padded so the record data starts on an even offset.
      let data_offset = record + 2 + key_len + key_len % 2;

      let data = catalog.get(data_offset..data_offset + 12).ok_or_else(|| {
        crate::Error::GenericError("catalog record points outside the fork".into())
      })?;
      if read_u16(&data[0..2]) == FOLDER_RECORD && read_u32(&data[8..12]) == ROOT_FOLDER_ID {
        return Ok(data_offset + FOLDER_FINDER_FLAGS);
      }
    }

    node = next;
  }

  Err(crate::Error::GenericError(
    "HFS+ catalog has no root folder record".into(),
  ))
}

#[cfg(test)]
/// Read the Finder flags of the volume root, for verification and tests.
pub fn root_folder_flags(image: &[u8]) -> crate::Result<u16> {
  let header = volume_header(image, Path::new("<memory>"))?;
  let block_size = read_u32(&header[40..44]) as usize;
  let extents = Fork::extents(header, CATALOG_FORK);
  let mut catalog = Vec::new();
  for (start, count) in extents {
    let start = start as usize * block_size;
    let end = start + count as usize * block_size;
    catalog.extend_from_slice(
      image
        .get(start..end)
        .ok_or_else(|| crate::Error::GenericError("catalog fork is truncated".into()))?,
    );
  }
  let offset = find_root_folder_flags(&catalog)?;
  Ok(read_u16(&catalog[offset..offset + 2]))
}

fn read_u16(bytes: &[u8]) -> u16 {
  u16::from_be_bytes([bytes[0], bytes[1]])
}

fn read_u32(bytes: &[u8]) -> u32 {
  u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Build a minimal catalog fork with one leaf node holding a single record.
  fn catalog(node_size: usize, record: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0u8; node_size * 2];
    // B-tree header: first leaf node = 1, node size.
    bytes[24..28].copy_from_slice(&1u32.to_be_bytes());
    bytes[32..34].copy_from_slice(&(node_size as u16).to_be_bytes());

    let node = node_size;
    bytes[node + 8] = 0xff;
    bytes[node + 10..node + 12].copy_from_slice(&1u16.to_be_bytes());
    let record_offset = 14;
    bytes[node + node_size - 2..node + node_size]
      .copy_from_slice(&(record_offset as u16).to_be_bytes());
    bytes[node + record_offset..node + record_offset + record.len()].copy_from_slice(record);
    bytes
  }

  /// Key: parent id 1, empty name; data: folder record for id 2.
  fn root_record() -> Vec<u8> {
    let mut record = vec![0u8; 2 + 6];
    record[0..2].copy_from_slice(&6u16.to_be_bytes());
    record[2..6].copy_from_slice(&1u32.to_be_bytes());
    record[6..8].copy_from_slice(&0u16.to_be_bytes());

    let mut data = vec![0u8; FOLDER_FINDER_FLAGS + 2];
    data[0..2].copy_from_slice(&FOLDER_RECORD.to_be_bytes());
    data[8..12].copy_from_slice(&ROOT_FOLDER_ID.to_be_bytes());
    record.extend_from_slice(&data);
    record
  }

  #[test]
  fn finds_the_root_folder_flags() {
    let catalog = catalog(512, &root_record());
    let offset = find_root_folder_flags(&catalog).unwrap();
    assert_eq!(offset, 512 + 14 + 8 + FOLDER_FINDER_FLAGS);
  }

  #[test]
  fn rejects_a_catalog_without_the_root_record() {
    let mut record = root_record();
    // Point the record at a different folder id.
    record[8 + 8..8 + 12].copy_from_slice(&7u32.to_be_bytes());
    let catalog = catalog(512, &record);
    assert!(find_root_folder_flags(&catalog).is_err());
  }

  #[test]
  fn reads_the_catalog_fork_description() {
    let mut header = vec![0u8; VOLUME_HEADER_LEN];
    header[0..2].copy_from_slice(b"H+");
    header[40..44].copy_from_slice(&4096u32.to_be_bytes());
    header[CATALOG_FORK + 16..CATALOG_FORK + 20].copy_from_slice(&96u32.to_be_bytes());
    header[CATALOG_FORK + 20..CATALOG_FORK + 24].copy_from_slice(&4u32.to_be_bytes());

    assert_eq!(Fork::extents(&header, CATALOG_FORK), [(96, 4)]);
  }
}
