#[warn(dead_code)]
use super::common;
use super::settings::Settings;
use super::wix;
use crate::ResultExt;
use std;
use std::collections::BTreeMap;

use std::path::PathBuf;

// Info about a resource file (including the main executable) in the bundle.
struct ResourceInfo {
  // The path to the existing file that will be bundled as a resource.
  source_path: PathBuf,
  // Relative path from the install dir where this will be installed.
  dest_path: PathBuf,
  // The name of this resource file in the filesystem.
  filename: String,
  // The size of this resource file, in bytes.
  size: u64,
  // The database key for the Component that this resource is part of.
  component_key: String,
}

// Info about a directory that needs to be created during installation.
struct DirectoryInfo {
  // The database key for this directory.
  key: String,
  // The database key for this directory's parent.
  parent_key: String,
  // The name of this directory in the filesystem.
  name: String,
  // List of files in this directory, not counting subdirectories.
  files: Vec<String>,
}

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  common::print_warning("MSI bundle support is still experimental.")?;

  let mut resources =
    collect_resource_info(settings).chain_err(|| "Failed to collect resource file information")?;
  let _directories = collect_directory_info(settings, &mut resources)
    .chain_err(|| "Failed to collect resource directory information")?;

  let wix_path = PathBuf::from("./WixTools");

  if !wix_path.exists() {
    wix::get_and_extract_wix(&wix_path)?;
  }

  let msi_path = wix::build_wix_app_installer(&settings, &wix_path)?;

  Ok(vec![msi_path])
}

// Returns a list of `ResourceInfo` structs for the binary executable and all
// the resource files that should be included in the package.
fn collect_resource_info(settings: &Settings) -> crate::Result<Vec<ResourceInfo>> {
  let mut resources = Vec::<ResourceInfo>::new();
  resources.push(ResourceInfo {
    source_path: settings.binary_path().to_path_buf(),
    dest_path: PathBuf::from(settings.binary_name()),
    filename: settings.binary_name().to_string(),
    size: settings.binary_path().metadata()?.len(),
    component_key: String::new(),
  });
  let root_rsrc_dir = PathBuf::from("Resources");
  for source_path in settings.resource_files() {
    let source_path = source_path?;
    let metadata = source_path.metadata()?;
    let size = metadata.len();
    let dest_path = root_rsrc_dir.join(common::resource_relpath(&source_path));
    let filename = dest_path.file_name().unwrap().to_string_lossy().to_string();
    let info = ResourceInfo {
      source_path,
      dest_path,
      filename,
      size,
      component_key: String::new(),
    };
    resources.push(info);
  }
  Ok(resources)
}

// Based on the list of all resource files to be bundled, returns a list of
// all the directories that need to be created during installation.  Also,
// modifies each `ResourceInfo` object to populate its `component_key` field
// with the database key of the Component that the resource will be associated
// with.
fn collect_directory_info(
  settings: &Settings,
  resources: &mut Vec<ResourceInfo>,
) -> crate::Result<Vec<DirectoryInfo>> {
  let mut dir_map = BTreeMap::<PathBuf, DirectoryInfo>::new();
  let mut dir_index: i32 = 0;
  dir_map.insert(
    PathBuf::new(),
    DirectoryInfo {
      key: "INSTALLDIR".to_string(),
      parent_key: "ProgramFilesFolder".to_string(),
      name: settings.bundle_name().to_string(),
      files: Vec::new(),
    },
  );
  for resource in resources.iter_mut() {
    let mut dir_key = "INSTALLDIR".to_string();
    let mut dir_path = PathBuf::new();
    for component in resource.dest_path.parent().unwrap().components() {
      if let std::path::Component::Normal(name) = component {
        dir_path.push(name);
        if dir_map.contains_key(&dir_path) {
          dir_key = dir_map.get(&dir_path).unwrap().key.clone();
        } else {
          let new_key = format!("RDIR{:04}", dir_index);
          dir_map.insert(
            dir_path.clone(),
            DirectoryInfo {
              key: new_key.clone(),
              parent_key: dir_key.clone(),
              name: name.to_string_lossy().to_string(),
              files: Vec::new(),
            },
          );
          dir_key = new_key;
          dir_index += 1;
        }
      }
    }
    let directory = dir_map.get_mut(&dir_path).unwrap();
    debug_assert_eq!(directory.key, dir_key);
    directory.files.push(resource.filename.clone());
    resource.component_key = dir_key.to_string();
  }
  Ok(dir_map.into_iter().map(|(_k, v)| v).collect())
}
