use super::category::AppCategory;
use clap::ArgMatches;
use glob;
use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use target_build_utils::TargetInfo;
use toml;
use walkdir;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageType {
  OsxBundle,
  #[cfg(feature = "ios")]
  IosBundle,
  #[cfg(feature = "wix")]
  WindowsMsi,
  Deb,
  Rpm,
  #[cfg(feature = "appimage")]
  AppImage,
  #[cfg(feature = "dmg")]
  Dmg,
}

impl PackageType {
  pub fn from_short_name(name: &str) -> Option<PackageType> {
    // Other types we may eventually want to support: apk
    match name {
      "deb" => Some(PackageType::Deb),
      #[cfg(feature = "ios")]
      "ios" => Some(PackageType::IosBundle),
      #[cfg(feature = "wix")]
      "msi" => Some(PackageType::WindowsMsi),
      "osx" => Some(PackageType::OsxBundle),
      "rpm" => Some(PackageType::Rpm),
      #[cfg(feature = "appimage")]
      "appimage" => Some(PackageType::AppImage),
      #[cfg(feature = "dmg")]
      "dmg" => Some(PackageType::Dmg),
      _ => None,
    }
  }

  pub fn short_name(&self) -> &'static str {
    match *self {
      PackageType::Deb => "deb",
      PackageType::IosBundle => "ios",
      PackageType::WindowsMsi => "msi",
      PackageType::OsxBundle => "osx",
      PackageType::Rpm => "rpm",
      PackageType::AppImage => "appimage",
      PackageType::Dmg => "dmg",
    }
  }

  pub fn all() -> &'static [PackageType] {
    ALL_PACKAGE_TYPES
  }
}

const ALL_PACKAGE_TYPES: &[PackageType] = &[
  PackageType::Deb,
  PackageType::IosBundle,
  PackageType::WindowsMsi,
  PackageType::OsxBundle,
  PackageType::Rpm,
];

#[derive(Clone, Debug)]
pub enum BuildArtifact {
  Main,
  Bin(String),
  Example(String),
}

#[derive(Clone, Debug, Deserialize)]
struct BundleSettings {
  // General settings:
  name: Option<String>,
  identifier: Option<String>,
  icon: Option<Vec<String>>,
  version: Option<String>,
  resources: Option<Vec<String>>,
  copyright: Option<String>,
  category: Option<AppCategory>,
  short_description: Option<String>,
  long_description: Option<String>,
  script: Option<PathBuf>,
  // OS-specific settings:
  deb_depends: Option<Vec<String>>,
  osx_frameworks: Option<Vec<String>>,
  osx_minimum_system_version: Option<String>,
  // Bundles for other binaries/examples:
  bin: Option<HashMap<String, BundleSettings>>,
  example: Option<HashMap<String, BundleSettings>>,
}

#[derive(Clone, Debug, Deserialize)]
struct MetadataSettings {
  bundle: Option<BundleSettings>,
}

#[derive(Clone, Debug, Deserialize)]
struct PackageSettings {
  name: String,
  version: String,
  description: String,
  homepage: Option<String>,
  authors: Option<Vec<String>>,
  metadata: Option<MetadataSettings>,
}

#[derive(Clone, Debug, Deserialize)]
struct WorkspaceSettings {
  members: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoSettings {
  package: Option<PackageSettings>, // "Ancestor" workspace Cargo.toml files may not have package info
  workspace: Option<WorkspaceSettings>, // "Ancestor" workspace Cargo.toml files may declare workspaces
}

#[derive(Clone, Debug)]
pub struct Settings {
  package: PackageSettings,
  package_type: Option<PackageType>, // If `None`, use the default package type for this os
  target: Option<(String, TargetInfo)>,
  features: Option<Vec<String>>,
  project_out_directory: PathBuf,
  build_artifact: BuildArtifact,
  is_release: bool,
  binary_path: PathBuf,
  binary_name: String,
  bundle_settings: BundleSettings,
}

impl CargoSettings {
  /*
      Try to load a set of CargoSettings from a "Cargo.toml" file in the specified directory
  */
  fn load(dir: &PathBuf) -> crate::Result<Self> {
    let toml_path = dir.join("Cargo.toml");
    let mut toml_str = String::new();
    let mut toml_file = File::open(toml_path)?;
    toml_file.read_to_string(&mut toml_str)?;
    toml::from_str(&toml_str).map_err(|e| e.into())
  }
}

impl Settings {
  pub fn new(current_dir: PathBuf, matches: &ArgMatches<'_>) -> crate::Result<Self> {
    let package_type = match matches.value_of("format") {
      Some(name) => match PackageType::from_short_name(name) {
        Some(package_type) => Some(package_type),
        None => bail!("Unsupported bundle format: {}", name),
      },
      None => None,
    };
    let build_artifact = if let Some(bin) = matches.value_of("bin") {
      BuildArtifact::Bin(bin.to_string())
    } else if let Some(example) = matches.value_of("example") {
      BuildArtifact::Example(example.to_string())
    } else {
      BuildArtifact::Main
    };
    let is_release = matches.is_present("release");
    let target = match matches.value_of("target") {
      Some(triple) => Some((triple.to_string(), TargetInfo::from_str(triple)?)),
      None => None,
    };
    let features = if matches.is_present("features") {
      Some(
        matches
          .values_of("features")
          .unwrap()
          .map(|s| s.to_string())
          .collect(),
      )
    } else {
      None
    };
    let cargo_settings = CargoSettings::load(&current_dir)?;
    let package = match cargo_settings.package {
      Some(package_info) => package_info,
      None => bail!("No 'package' info found in 'Cargo.toml'"),
    };
    let workspace_dir = Settings::get_workspace_dir(&current_dir);
    let target_dir = Settings::get_target_dir(&workspace_dir, &target, is_release, &build_artifact);
    let bundle_settings = if let Some(bundle_settings) = package
      .metadata
      .as_ref()
      .and_then(|metadata| metadata.bundle.as_ref())
    {
      bundle_settings.clone()
    } else {
      bail!("No [package.metadata.bundle] section in Cargo.toml");
    };
    let (bundle_settings, binary_name) = match build_artifact {
      BuildArtifact::Main => (bundle_settings, package.name.clone()),
      BuildArtifact::Bin(ref name) => (
        bundle_settings_from_table(&bundle_settings.bin, "bin", name)?,
        name.clone(),
      ),
      BuildArtifact::Example(ref name) => (
        bundle_settings_from_table(&bundle_settings.example, "example", name)?,
        name.clone(),
      ),
    };
    let binary_name = if cfg!(windows) {
      format!("{}.{}", &binary_name, "exe")
    } else {
      binary_name
    };
    let binary_path = target_dir.join(&binary_name);
    Ok(Settings {
      package,
      package_type,
      target,
      features,
      build_artifact,
      is_release,
      project_out_directory: target_dir,
      binary_path,
      binary_name,
      bundle_settings,
    })
  }

  /*
      The target_dir where binaries will be compiled to by cargo can vary:
          - this directory is a member of a workspace project
          - overridden by CARGO_TARGET_DIR environment variable
          - specified in build.target-dir configuration key
          - if the build is a 'release' or 'debug' build

      This function determines where 'target' dir is and suffixes it with 'release' or 'debug'
      to determine where the compiled binary will be located.
  */
  fn get_target_dir(
    project_root_dir: &PathBuf,
    target: &Option<(String, TargetInfo)>,
    is_release: bool,
    build_artifact: &BuildArtifact,
  ) -> PathBuf {
    let mut path = project_root_dir.join("target");
    if let &Some((ref triple, _)) = target {
      path.push(triple);
    }
    path.push(if is_release { "release" } else { "debug" });
    if let &BuildArtifact::Example(_) = build_artifact {
      path.push("examples");
    }
    path
  }

  /*
      The specification of the Cargo.toml Manifest that covers the "workspace" section is here:
      https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-section

      Determining if the current project folder is part of a workspace:
          - Walk up the file system, looking for a Cargo.toml file.
          - Stop at the first one found.
          - If one is found before reaching "/" then this folder belongs to that parent workspace,
            if it contains a [workspace] entry and the project crate name is listed on the "members" array
  */
  pub fn get_workspace_dir(current_dir: &PathBuf) -> PathBuf {
    let mut dir = current_dir.clone();
    let project_name = CargoSettings::load(&dir).unwrap().package.unwrap().name;

    while dir.pop() {
      match CargoSettings::load(&dir) {
        Ok(cargo_settings) => match cargo_settings.workspace {
          Some(workspace_settings) => {
            if workspace_settings.members.is_some()
              && workspace_settings
                .members
                .unwrap()
                .iter()
                .any(|member| member.as_str() == project_name)
            {
              return dir;
            }
          }
          None => {}
        },
        Err(_) => {}
      }
    }

    // Nothing found walking up the file system, return the starting directory
    current_dir.clone()
  }

  /// Returns the directory where the bundle should be placed.
  pub fn project_out_directory(&self) -> &Path {
    &self.project_out_directory
  }

  /// Returns the architecture for the binary being bundled (e.g. "arm" or
  /// "x86" or "x86_64").
  pub fn binary_arch(&self) -> &str {
    if let Some((_, ref info)) = self.target {
      info.target_arch()
    } else {
      std::env::consts::ARCH
    }
  }

  /// Returns the file name of the binary being bundled.
  pub fn binary_name(&self) -> &str {
    &self.binary_name
  }

  /// Returns the path to the binary being bundled.
  pub fn binary_path(&self) -> &Path {
    &self.binary_path
  }

  /// If a specific package type was specified by the command-line, returns
  /// that package type; otherwise, if a target triple was specified by the
  /// command-line, returns the native package type(s) for that target;
  /// otherwise, returns the native package type(s) for the host platform.
  /// Fails if the host/target's native package type is not supported.
  pub fn package_types(&self) -> crate::Result<Vec<PackageType>> {
    if let Some(package_type) = self.package_type {
      Ok(vec![package_type])
    } else {
      let target_os = if let Some((_, ref info)) = self.target {
        info.target_os()
      } else {
        std::env::consts::OS
      };
      match target_os {
        "macos" => Ok(vec![PackageType::OsxBundle]),
        "ios" => Ok(vec![PackageType::IosBundle]),
        "linux" => Ok(vec![PackageType::Deb]), // TODO: Do Rpm too, once it's implemented.
        "windows" => Ok(vec![PackageType::WindowsMsi]),
        os => bail!("Native {} bundles not yet supported.", os),
      }
    }
  }

  /// If the bundle is being cross-compiled, returns the target triple string
  /// (e.g. `"x86_64-apple-darwin"`).  If the bundle is targeting the host
  /// environment, returns `None`.
  pub fn target_triple(&self) -> Option<&str> {
    match self.target {
      Some((ref triple, _)) => Some(triple.as_str()),
      None => None,
    }
  }

  /// Returns the features that is being built.
  pub fn build_features(&self) -> Option<Vec<String>> {
    self.features.to_owned()
  }

  /// Returns the artifact that is being bundled.
  pub fn build_artifact(&self) -> &BuildArtifact {
    &self.build_artifact
  }

  /// Returns true if the bundle is being compiled in release mode, false if
  /// it's being compiled in debug mode.
  pub fn is_release_build(&self) -> bool {
    self.is_release
  }

  pub fn bundle_name(&self) -> &str {
    self
      .bundle_settings
      .name
      .as_ref()
      .unwrap_or(&self.package.name)
  }

  pub fn bundle_identifier(&self) -> &str {
    self
      .bundle_settings
      .identifier
      .as_ref()
      .map(String::as_str)
      .unwrap_or("")
  }

  /// Returns an iterator over the icon files to be used for this bundle.
  pub fn icon_files(&self) -> ResourcePaths<'_> {
    match self.bundle_settings.icon {
      Some(ref paths) => ResourcePaths::new(paths.as_slice(), false),
      None => ResourcePaths::new(&[], false),
    }
  }

  /// Returns an iterator over the resource files to be included in this
  /// bundle.
  pub fn resource_files(&self) -> ResourcePaths<'_> {
    match self.bundle_settings.resources {
      Some(ref paths) => ResourcePaths::new(paths.as_slice(), true),
      None => ResourcePaths::new(&[], true),
    }
  }

  pub fn version_string(&self) -> &str {
    self
      .bundle_settings
      .version
      .as_ref()
      .unwrap_or(&self.package.version)
  }

  pub fn copyright_string(&self) -> Option<&str> {
    self.bundle_settings.copyright.as_ref().map(String::as_str)
  }

  pub fn author_names(&self) -> &[String] {
    match self.package.authors {
      Some(ref names) => names.as_slice(),
      None => &[],
    }
  }

  pub fn authors_comma_separated(&self) -> Option<String> {
    let names = self.author_names();
    if names.is_empty() {
      None
    } else {
      Some(names.join(", "))
    }
  }

  pub fn homepage_url(&self) -> &str {
    &self
      .package
      .homepage
      .as_ref()
      .map(String::as_str)
      .unwrap_or("")
  }

  pub fn app_category(&self) -> Option<AppCategory> {
    self.bundle_settings.category
  }

  pub fn short_description(&self) -> &str {
    self
      .bundle_settings
      .short_description
      .as_ref()
      .unwrap_or(&self.package.description)
  }

  pub fn long_description(&self) -> Option<&str> {
    self
      .bundle_settings
      .long_description
      .as_ref()
      .map(String::as_str)
  }

  pub fn debian_dependencies(&self) -> &[String] {
    match self.bundle_settings.deb_depends {
      Some(ref dependencies) => dependencies.as_slice(),
      None => &[],
    }
  }

  pub fn osx_frameworks(&self) -> &[String] {
    match self.bundle_settings.osx_frameworks {
      Some(ref frameworks) => frameworks.as_slice(),
      None => &[],
    }
  }

  pub fn osx_minimum_system_version(&self) -> Option<&str> {
    self
      .bundle_settings
      .osx_minimum_system_version
      .as_ref()
      .map(String::as_str)
  }
}

fn bundle_settings_from_table(
  opt_map: &Option<HashMap<String, BundleSettings>>,
  map_name: &str,
  bundle_name: &str,
) -> crate::Result<BundleSettings> {
  if let Some(bundle_settings) = opt_map.as_ref().and_then(|map| map.get(bundle_name)) {
    Ok(bundle_settings.clone())
  } else {
    bail!(
      "No [package.metadata.bundle.{}.{}] section in Cargo.toml",
      map_name,
      bundle_name
    );
  }
}

pub struct ResourcePaths<'a> {
  pattern_iter: std::slice::Iter<'a, String>,
  glob_iter: Option<glob::Paths>,
  walk_iter: Option<walkdir::IntoIter>,
  allow_walk: bool,
}

impl<'a> ResourcePaths<'a> {
  fn new(patterns: &'a [String], allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      pattern_iter: patterns.iter(),
      glob_iter: None,
      walk_iter: None,
      allow_walk,
    }
  }
}

impl<'a> Iterator for ResourcePaths<'a> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    loop {
      if let Some(ref mut walk_entries) = self.walk_iter {
        if let Some(entry) = walk_entries.next() {
          let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(Err(crate::Error::from(error))),
          };
          let path = entry.path();
          if path.is_dir() {
            continue;
          }
          return Some(Ok(path.to_path_buf()));
        }
      }
      self.walk_iter = None;
      if let Some(ref mut glob_paths) = self.glob_iter {
        if let Some(glob_result) = glob_paths.next() {
          let path = match glob_result {
            Ok(path) => path,
            Err(error) => return Some(Err(crate::Error::from(error))),
          };
          if path.is_dir() {
            if self.allow_walk {
              let walk = walkdir::WalkDir::new(path);
              self.walk_iter = Some(walk.into_iter());
              continue;
            } else {
              let msg = format!("{:?} is a directory", path);
              return Some(Err(crate::Error::from(msg)));
            }
          }
          return Some(Ok(path));
        }
      }
      self.glob_iter = None;
      if let Some(pattern) = self.pattern_iter.next() {
        let glob = match glob::glob(pattern) {
          Ok(glob) => glob,
          Err(error) => return Some(Err(crate::Error::from(error))),
        };
        self.glob_iter = Some(glob);
        continue;
      }
      return None;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{AppCategory, BundleSettings, CargoSettings};
  use toml;

  #[test]
  fn parse_cargo_toml() {
    let toml_str = "\
                    [package]\n\
                    name = \"example\"\n\
                    version = \"0.1.0\"\n\
                    authors = [\"Jane Doe\"]\n\
                    license = \"MIT\"\n\
                    description = \"An example application.\"\n\
                    build = \"build.rs\"\n\
                    \n\
                    [package.metadata.bundle]\n\
                    name = \"Example Application\"\n\
                    identifier = \"com.example.app\"\n\
                    resources = [\"data\", \"foo/bar\"]\n\
                    category = \"Puzzle Game\"\n\
                    long_description = \"\"\"\n\
                    This is an example of a\n\
                    simple application.\n\
                    \"\"\"\n\
                    \n\
                    [dependencies]\n\
                    rand = \"0.4\"\n";
    let cargo_settings: CargoSettings = toml::from_str(toml_str).unwrap();
    let package = cargo_settings.package.unwrap();
    assert_eq!(package.name, "example");
    assert_eq!(package.version, "0.1.0");
    assert_eq!(package.description, "An example application.");
    assert_eq!(package.homepage, None);
    assert_eq!(package.authors, Some(vec!["Jane Doe".to_string()]));
    assert!(package.metadata.is_some());
    let metadata = package.metadata.as_ref().unwrap();
    assert!(metadata.bundle.is_some());
    let bundle = metadata.bundle.as_ref().unwrap();
    assert_eq!(bundle.name, Some("Example Application".to_string()));
    assert_eq!(bundle.identifier, Some("com.example.app".to_string()));
    assert_eq!(bundle.icon, None);
    assert_eq!(bundle.version, None);
    assert_eq!(
      bundle.resources,
      Some(vec!["data".to_string(), "foo/bar".to_string()])
    );
    assert_eq!(bundle.category, Some(AppCategory::PuzzleGame));
    assert_eq!(
      bundle.long_description,
      Some(
        "This is an example of a\n\
         simple application.\n"
          .to_string()
      )
    );
  }

  #[test]
  fn parse_bin_and_example_bundles() {
    let toml_str = "\
            [package]\n\
            name = \"example\"\n\
            version = \"0.1.0\"\n\
            description = \"An example application.\"\n\
            \n\
            [package.metadata.bundle.bin.foo]\n\
            name = \"Foo App\"\n\
            \n\
            [package.metadata.bundle.bin.bar]\n\
            name = \"Bar App\"\n\
            \n\
            [package.metadata.bundle.example.baz]\n\
            name = \"Baz Example\"\n\
            \n\
            [[bin]]\n\
            name = \"foo\"\n
            \n\
            [[bin]]\n\
            name = \"bar\"\n
            \n\
            [[example]]\n\
            name = \"baz\"\n";
    let cargo_settings: CargoSettings = toml::from_str(toml_str).unwrap();
    assert!(cargo_settings.package.is_some());
    let package = cargo_settings.package.as_ref().unwrap();
    assert!(package.metadata.is_some());
    let metadata = package.metadata.as_ref().unwrap();
    assert!(metadata.bundle.is_some());
    let bundle = metadata.bundle.as_ref().unwrap();
    assert!(bundle.example.is_some());

    let bins = bundle.bin.as_ref().unwrap();
    assert!(bins.contains_key("foo"));
    let foo: &BundleSettings = bins.get("foo").unwrap();
    assert_eq!(foo.name, Some("Foo App".to_string()));
    assert!(bins.contains_key("bar"));
    let bar: &BundleSettings = bins.get("bar").unwrap();
    assert_eq!(bar.name, Some("Bar App".to_string()));

    let examples = bundle.example.as_ref().unwrap();
    assert!(examples.contains_key("baz"));
    let baz: &BundleSettings = examples.get("baz").unwrap();
    assert_eq!(baz.name, Some("Baz Example".to_string()));
  }
}
