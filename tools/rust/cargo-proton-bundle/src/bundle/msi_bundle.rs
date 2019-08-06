use super::common;
use super::settings::Settings;
use super::wix;
use crate::ResultExt;
use cab;
use dirs;
use msi;
use slog::Drain;
use slog_term;
use std;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

type Package = msi::Package<fs::File>;

// Don't add more files to a cabinet folder that already has this many bytes:
const CABINET_FOLDER_SIZE_LIMIT: u64 = 0x8000;
// The maximum number of resource files we'll put in one cabinet:
const CABINET_MAX_FILES: usize = 1000;
// The maximum number of data bytes we'll put in one cabinet:
const CABINET_MAX_SIZE: u64 = 0x1000_0000;

// File table attribute indicating that a file is "vital":
const FILE_ATTR_VITAL: u16 = 0x200;

// The name of the installer package's sole Feature:
const MAIN_FEATURE_NAME: &str = "MainFeature";

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

// Info about a CAB archive within the installer package.
struct CabinetInfo {
  // The stream name for this cabinet.
  name: String,
  // The resource files that are in this cabinet.
  resources: Vec<ResourceInfo>,
}

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  common::print_warning("MSI bundle support is still experimental.")?;

  let msi_name = format!("{}.msi", settings.bundle_name());
  common::print_bundling(&msi_name)?;
  let base_dir = settings.project_out_directory().join("bundle/msi");
  let msi_path = base_dir.join(&msi_name);
  let mut package =
    new_empty_package(&msi_path).chain_err(|| "Failed to initialize MSI package")?;

  // Generate package metadata:
  // let guid = generate_package_guid(settings);
  // set_summary_info(&mut package, guid, settings);
  // create_property_table(&mut package, guid, settings)
  //   .chain_err(|| "Failed to generate Property table")?;

  // Copy resource files into package:
  let mut resources =
    collect_resource_info(settings).chain_err(|| "Failed to collect resource file information")?;
  let _directories = collect_directory_info(settings, &mut resources)
    .chain_err(|| "Failed to collect resource directory information")?;
  let cabinets = divide_resources_into_cabinets(resources);
  generate_resource_cabinets(&mut package, &cabinets)
    .chain_err(|| "Failed to generate resource cabinets")?;
  let decorator = slog_term::TermDecorator::new().build();
  let drain = slog_term::CompactFormat::new(decorator).build();
  let drain = std::sync::Mutex::new(drain).fuse();
  let logger = slog::Logger::root(drain, o!());
  let wix_path = PathBuf::from("./WixTools");

  wix::get_and_extract_wix(&logger, &wix_path)?;

  wix::build_wix_app_installer(&logger, &settings, &wix_path, PathBuf::from("."))?;

  // Set up installer database tables:
  // create_directory_table(&mut package, &directories)
  //   .chain_err(|| "Failed to generate Directory table")?;
  // create_feature_table(&mut package, settings).chain_err(|| "Failed to generate Feature table")?;
  // create_component_table(&mut package, guid, &directories)
  //   .chain_err(|| "Failed to generate Component table")?;
  // create_feature_components_table(&mut package, &directories)
  //   .chain_err(|| "Failed to generate FeatureComponents table")?;
  // create_media_table(&mut package, &cabinets).chain_err(|| "Failed to generate Media table")?;
  // create_file_table(&mut package, &cabinets).chain_err(|| "Failed to generate File table")?;
  // // TODO: Create other needed tables.

  // // Create app icon:
  // package.create_table(
  //   "Icon",
  //   vec![
  //     msi::Column::build("Name").primary_key().id_string(72),
  //     msi::Column::build("Data").binary(),
  //   ],
  // )?;
  // let icon_name = format!("{}.ico", settings.binary_name());
  // {
  //   let stream_name = format!("Icon.{}", icon_name);
  //   let mut stream = package.write_stream(&stream_name)?;
  //   create_app_icon(&mut stream, settings)?;
  // }
  // package.insert_rows(msi::Insert::into("Icon").row(vec![
  //   msi::Value::Str(icon_name.clone()),
  //   msi::Value::from("Name"),
  // ]))?;

  // package.flush()?;
  Ok(vec![msi_path])
}

fn new_empty_package(msi_path: &Path) -> crate::Result<Package> {
  if let Some(parent) = msi_path.parent() {
    fs::create_dir_all(&parent).chain_err(|| format!("Failed to create directory {:?}", parent))?;
  }
  let msi_file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(msi_path)
    .chain_err(|| format!("Failed to create file {:?}", msi_path))?;
  let package = msi::Package::create(msi::PackageType::Installer, msi_file)?;
  Ok(package)
}

// Populates the summary metadata for the package from the bundle settings.
fn set_summary_info(package: &mut Package, package_guid: Uuid, settings: &Settings) {
  let summary_info = package.summary_info_mut();
  summary_info.set_creation_time_to_now();
  summary_info.set_subject(settings.bundle_name().to_string());
  summary_info.set_uuid(package_guid);
  summary_info.set_comments(settings.short_description().to_string());
  if let Some(authors) = settings.authors_comma_separated() {
    summary_info.set_author(authors);
  }
  let creating_app = format!("cargo-bundle v{}", crate_version!());
  summary_info.set_creating_application(creating_app);
}

// Creates and populates the `Property` database table for the package.
fn create_property_table(
  package: &mut Package,
  package_guid: Uuid,
  settings: &Settings,
) -> crate::Result<()> {
  let authors = settings.authors_comma_separated().unwrap_or(String::new());
  package.create_table(
    "Property",
    vec![
      msi::Column::build("Property").primary_key().id_string(72),
      msi::Column::build("Value").text_string(0),
    ],
  )?;
  package.insert_rows(
    msi::Insert::into("Property")
      .row(vec![
        msi::Value::from("Manufacturer"),
        msi::Value::Str(authors),
      ])
      .row(vec![
        msi::Value::from("ProductCode"),
        msi::Value::from(package_guid),
      ])
      .row(vec![
        msi::Value::from("ProductLanguage"),
        msi::Value::from(msi::Language::from_tag("en-US")),
      ])
      .row(vec![
        msi::Value::from("ProductName"),
        msi::Value::from(settings.bundle_name()),
      ])
      .row(vec![
        msi::Value::from("ProductVersion"),
        msi::Value::from(settings.version_string()),
      ]),
  )?;
  Ok(())
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

// Divides up the list of resource into some number of cabinets, subject to a
// few contraints: 1) no one cabinet will have two resources with the same
// filename, 2) no one cabinet will have more than `CABINET_MAX_FILES` files
// in it, and 3) no one cabinet will contain mroe than `CABINET_MAX_SIZE`
// bytes of data (unless that cabinet consists of a single file that is
// already bigger than that).
fn divide_resources_into_cabinets(mut resources: Vec<ResourceInfo>) -> Vec<CabinetInfo> {
  let mut cabinets = Vec::new();
  while !resources.is_empty() {
    let mut filenames = HashSet::<String>::new();
    let mut total_size = 0;
    let mut leftovers = Vec::<ResourceInfo>::new();
    let mut cabinet = CabinetInfo {
      name: format!("rsrc{:04}.cab", cabinets.len()),
      resources: Vec::new(),
    };
    for resource in resources.into_iter() {
      if cabinet.resources.len() >= CABINET_MAX_FILES
        || (!cabinet.resources.is_empty() && total_size + resource.size > CABINET_MAX_SIZE)
        || filenames.contains(&resource.filename)
      {
        leftovers.push(resource);
      } else {
        filenames.insert(resource.filename.clone());
        total_size += resource.size;
        cabinet.resources.push(resource);
      }
    }
    cabinets.push(cabinet);
    resources = leftovers;
  }
  cabinets
}

// Creates the CAB archives within the package that contain the binary
// execuable and all the resource files.
fn generate_resource_cabinets(
  package: &mut Package,
  cabinets: &[CabinetInfo],
) -> crate::Result<()> {
  for cabinet_info in cabinets.iter() {
    let mut builder = cab::CabinetBuilder::new();
    let mut file_map = HashMap::<String, &Path>::new();
    let mut resource_index: usize = 0;
    while resource_index < cabinet_info.resources.len() {
      let folder = builder.add_folder(cab::CompressionType::MsZip);
      let mut folder_size: u64 = 0;
      while resource_index < cabinet_info.resources.len() && folder_size < CABINET_FOLDER_SIZE_LIMIT
      {
        let resource = &cabinet_info.resources[resource_index];
        folder_size += resource.size;
        folder.add_file(resource.filename.as_str());
        debug_assert!(!file_map.contains_key(&resource.filename));
        file_map.insert(resource.filename.clone(), &resource.source_path);
        resource_index += 1;
      }
    }
    let stream = package.write_stream(cabinet_info.name.as_str())?;
    let mut cabinet_writer = builder.build(stream)?;
    while let Some(mut file_writer) = cabinet_writer.next_file()? {
      debug_assert!(file_map.contains_key(file_writer.file_name()));
      let file_path = file_map.get(file_writer.file_name()).unwrap();
      let mut file = fs::File::open(file_path)?;
      io::copy(&mut file, &mut file_writer)?;
    }
    cabinet_writer.finish()?;
  }
  Ok(())
}

// Creates and populates the `Directory` database table for the package.
fn create_directory_table(
  package: &mut Package,
  directories: &[DirectoryInfo],
) -> crate::Result<()> {
  package.create_table(
    "Directory",
    vec![
      msi::Column::build("Directory").primary_key().id_string(72),
      msi::Column::build("Directory_Parent")
        .nullable()
        .foreign_key("Directory", 1)
        .id_string(72),
      msi::Column::build("DefaultDir")
        .category(msi::Category::DefaultDir)
        .string(255),
    ],
  )?;
  let mut rows = Vec::new();
  for directory in directories.iter() {
    rows.push(vec![
      msi::Value::Str(directory.key.clone()),
      msi::Value::Str(directory.parent_key.clone()),
      msi::Value::Str(directory.name.clone()),
    ]);
  }
  package.insert_rows(
    msi::Insert::into("Directory")
      .row(vec![
        msi::Value::from("TARGETDIR"),
        msi::Value::Null,
        msi::Value::from("SourceDir"),
      ])
      .row(vec![
        msi::Value::from("ProgramFilesFolder"),
        msi::Value::from("TARGETDIR"),
        msi::Value::from("."),
      ])
      .rows(rows),
  )?;
  Ok(())
}

// Creates and populates the `Feature` database table for the package.  The
// package will have a single main feature that installs everything.
fn create_feature_table(package: &mut Package, settings: &Settings) -> crate::Result<()> {
  package.create_table(
    "Feature",
    vec![
      msi::Column::build("Feature").primary_key().id_string(38),
      msi::Column::build("Feature_Parent")
        .nullable()
        .foreign_key("Feature", 1)
        .id_string(38),
      msi::Column::build("Title").nullable().text_string(64),
      msi::Column::build("Description")
        .nullable()
        .text_string(255),
      msi::Column::build("Display")
        .nullable()
        .range(0, 0x7fff)
        .int16(),
      msi::Column::build("Level").range(0, 0x7fff).int16(),
      msi::Column::build("Directory_")
        .nullable()
        .foreign_key("Directory", 1)
        .id_string(72),
      msi::Column::build("Attributes").int16(),
    ],
  )?;
  package.insert_rows(msi::Insert::into("Feature").row(vec![
    msi::Value::from(MAIN_FEATURE_NAME),
    msi::Value::Null,
    msi::Value::from(settings.bundle_name()),
    msi::Value::Null,
    msi::Value::Int(1),
    msi::Value::Int(3),
    msi::Value::from("INSTALLDIR"),
    msi::Value::Int(0),
  ]))?;
  Ok(())
}

// Creates and populates the `Component` database table for the package.  One
// component is created for each subdirectory under in the install dir.
fn create_component_table(
  package: &mut Package,
  package_guid: Uuid,
  directories: &[DirectoryInfo],
) -> crate::Result<()> {
  package.create_table(
    "Component",
    vec![
      msi::Column::build("Component").primary_key().id_string(72),
      msi::Column::build("ComponentId")
        .nullable()
        .category(msi::Category::Guid)
        .string(38),
      msi::Column::build("Directory_")
        .nullable()
        .foreign_key("Directory", 1)
        .id_string(72),
      msi::Column::build("Attributes").int16(),
      msi::Column::build("Condition")
        .nullable()
        .category(msi::Category::Condition)
        .string(255),
      msi::Column::build("KeyPath").nullable().id_string(72),
    ],
  )?;
  let mut rows = Vec::new();
  for directory in directories.iter() {
    if !directory.files.is_empty() {
      let hash_input = directory.files.join("/");
      rows.push(vec![
        msi::Value::Str(directory.key.clone()),
        msi::Value::from(Uuid::new_v5(&package_guid, &hash_input)),
        msi::Value::Str(directory.key.clone()),
        msi::Value::Int(0),
        msi::Value::Null,
        msi::Value::Str(directory.files[0].clone()),
      ]);
    }
  }
  package.insert_rows(msi::Insert::into("Component").rows(rows))?;
  Ok(())
}

// Creates and populates the `FeatureComponents` database table for the
// package.  All components are added to the package's single main feature.
fn create_feature_components_table(
  package: &mut Package,
  directories: &[DirectoryInfo],
) -> crate::Result<()> {
  package.create_table(
    "FeatureComponents",
    vec![
      msi::Column::build("Feature_")
        .primary_key()
        .foreign_key("Component", 1)
        .id_string(38),
      msi::Column::build("Component_")
        .primary_key()
        .foreign_key("Component", 1)
        .id_string(72),
    ],
  )?;
  let mut rows = Vec::new();
  for directory in directories.iter() {
    if !directory.files.is_empty() {
      rows.push(vec![
        msi::Value::from(MAIN_FEATURE_NAME),
        msi::Value::Str(directory.key.clone()),
      ]);
    }
  }
  package.insert_rows(msi::Insert::into("FeatureComponents").rows(rows))?;
  Ok(())
}

// Creates and populates the `Media` database table for the package, with one
// entry for each CAB archive within the package.
fn create_media_table(package: &mut Package, cabinets: &[CabinetInfo]) -> crate::Result<()> {
  package.create_table(
    "Media",
    vec![
      msi::Column::build("DiskId")
        .primary_key()
        .range(1, 0x7fff)
        .int16(),
      msi::Column::build("LastSequence").range(0, 0x7fff).int16(),
      msi::Column::build("DiskPrompt").nullable().text_string(64),
      msi::Column::build("Cabinet")
        .nullable()
        .category(msi::Category::Cabinet)
        .string(255),
      msi::Column::build("VolumeLabel").nullable().text_string(32),
      msi::Column::build("Source")
        .nullable()
        .category(msi::Category::Property)
        .string(32),
    ],
  )?;
  let mut disk_id: i32 = 0;
  let mut last_seq: i32 = 0;
  let mut rows = Vec::new();
  for cabinet in cabinets.iter() {
    disk_id += 1;
    last_seq += cabinet.resources.len() as i32;
    rows.push(vec![
      msi::Value::Int(disk_id),
      msi::Value::Int(last_seq),
      msi::Value::Null,
      msi::Value::Str(format!("#{}", cabinet.name)),
      msi::Value::Null,
      msi::Value::Null,
    ]);
  }
  package.insert_rows(msi::Insert::into("Media").rows(rows))?;
  Ok(())
}

// Creates and populates the `File` database table for the package, with one
// entry for each resource file to be installed (including the main
// executable).
fn create_file_table(package: &mut Package, cabinets: &[CabinetInfo]) -> crate::Result<()> {
  package.create_table(
    "File",
    vec![
      msi::Column::build("File").primary_key().id_string(72),
      msi::Column::build("Component_")
        .foreign_key("Component", 1)
        .id_string(72),
      msi::Column::build("FileName")
        .category(msi::Category::Filename)
        .string(255),
      msi::Column::build("FileSize").range(0, 0x7fffffff).int32(),
      msi::Column::build("Version")
        .nullable()
        .category(msi::Category::Version)
        .string(72),
      msi::Column::build("Language")
        .nullable()
        .category(msi::Category::Language)
        .string(20),
      msi::Column::build("Attributes")
        .nullable()
        .range(0, 0x7fff)
        .int16(),
      msi::Column::build("Sequence").range(1, 0x7fff).int16(),
    ],
  )?;
  let mut rows = Vec::new();
  let mut sequence: i32 = 1;
  for cabinet in cabinets.iter() {
    for resource in cabinet.resources.iter() {
      rows.push(vec![
        msi::Value::Str(format!("r{:04}", sequence)),
        msi::Value::Str(resource.component_key.clone()),
        msi::Value::Str(resource.filename.clone()),
        msi::Value::Int(resource.size as i32),
        msi::Value::Null,
        msi::Value::Null,
        msi::Value::from(FILE_ATTR_VITAL),
        msi::Value::Int(sequence),
      ]);
      sequence += 1;
    }
  }
  package.insert_rows(msi::Insert::into("File").rows(rows))?;
  Ok(())
}

fn create_app_icon<W: Write>(writer: &mut W, settings: &Settings) -> crate::Result<()> {
  // Prefer ICO files.
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() == Some(OsStr::new("ico")) {
      io::copy(&mut fs::File::open(icon_path)?, writer)?;
      return Ok(());
    }
  }
  // TODO: Convert from other formats.
  Ok(())
}
