// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::debian::{generate_data, tar_and_gzip_dir};
use crate::{
  Settings,
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::fs_utils,
};
use flate2::Compression;
use heck::AsKebabCase;
use sha2::{Digest, Sha512};

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Returns the paths of the generated `.tar.gz` archive, its `PKGBUILD` and `.SRCINFO`.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // Arch Linux uses its own naming for some architectures.
  let arch = match settings.binary_arch() {
    Arch::X86_64 => "x86_64",
    Arch::X86 => "i686",
    Arch::AArch64 => "aarch64",
    Arch::Armhf => "armv7h",
    Arch::Armel => "arm",
    Arch::Riscv64 => "riscv64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };
  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    arch
  );

  let base_dir = settings.project_out_directory().join("bundle/pacman");

  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).fs_context(
      "Failed to Remove old package directory",
      package_dir.clone(),
    )?;
  }

  let package_name = format!("{package_base_name}.tar.gz");
  let package_path = base_dir.join(&package_name);
  log::info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  let (data_dir, _) =
    generate_data(settings, &package_dir).context("Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.deb().files, &data_dir)
    .context("Failed to copy custom files")?;

  // Apply tar/gzip to create the final package file.
  let package_path = tar_and_gzip_dir(package_dir, Compression::default())
    .context("Failed to create the tar.gz package")?;

  let package = Package::new(settings, arch, &package_path)?;

  if settings
    .pacman()
    .source
    .as_ref()
    .is_none_or(|s| s.is_empty())
  {
    log::info!(
      "The PKGBUILD refers to the local archive. To publish it to the AUR, upload the archive and set `bundle > linux > pacman > source` to its download URL."
    );
  }

  let pkgbuild_path = base_dir.join("PKGBUILD");
  write_file(&pkgbuild_path, &package.pkgbuild()).context("Failed to create PKGBUILD file")?;
  let srcinfo_path = base_dir.join(".SRCINFO");
  write_file(&srcinfo_path, &package.srcinfo()).context("Failed to create .SRCINFO file")?;

  Ok(vec![package_path, pkgbuild_path, srcinfo_path])
}

fn write_file(path: &Path, contents: &str) -> crate::Result<()> {
  let mut file = fs_utils::create_file(path)?;
  file.write_all(contents.as_bytes())?;
  file.flush()?;
  Ok(())
}

/// The package metadata, rendered both as a PKGBUILD and as the .SRCINFO the AUR requires.
///
/// See <https://wiki.archlinux.org/title/PKGBUILD> and <https://wiki.archlinux.org/title/.SRCINFO>
struct Package {
  maintainer: String,
  pkgname: String,
  /// The app version, as is, available to `source` as `${_pkgver}`.
  version: String,
  pkgver: String,
  pkgrel: String,
  epoch: Option<u32>,
  pkgdesc: String,
  arch: String,
  url: String,
  license: String,
  depends: Vec<String>,
  optdepends: Vec<String>,
  provides: Vec<String>,
  conflicts: Vec<String>,
  replaces: Vec<String>,
  source: Vec<String>,
  sha512sums: Vec<String>,
}

impl Package {
  fn new(settings: &Settings, arch: &str, package_path: &Path) -> crate::Result<Self> {
    let pacman = settings.pacman();
    let archive_name = package_path
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .into_owned();

    let pkgname = match &pacman.pkgname {
      Some(pkgname) => {
        if !is_valid_pkgname(pkgname) {
          return Err(crate::Error::GenericError(format!(
            "invalid `bundle > linux > pacman > pkgname` `{pkgname}`: it must only contain lowercase alphanumerics and `@._+-`, and must not start with `-` or `.`"
          )));
        }
        pkgname.clone()
      }
      None => default_pkgname(settings.product_name()).ok_or_else(|| {
        crate::Error::GenericError(format!(
          "cannot derive a pacman package name from the product name `{}`, set `bundle > linux > pacman > pkgname`",
          settings.product_name()
        ))
      })?,
    };

    let pkgrel = pacman.pkgrel.clone().unwrap_or_else(|| "1".into());
    if !is_valid_pkgrel(&pkgrel) {
      return Err(crate::Error::GenericError(format!(
        "invalid `bundle > linux > pacman > pkgrel` `{pkgrel}`: it must be a positive integer, optionally followed by a period and another integer, e.g. `1` or `2.1`"
      )));
    }

    let mut provides = pacman.provides.clone().unwrap_or_default();
    let mut conflicts = pacman.conflicts.clone().unwrap_or_default();
    // a prebuilt `-bin` package stands in for the package built from source
    if let Some(name) = pkgname.strip_suffix("-bin").filter(|n| !n.is_empty()) {
      for list in [&mut provides, &mut conflicts] {
        if !list.iter().any(|p| p == name) {
          list.push(name.to_string());
        }
      }
    }

    let mut source = pacman.source.clone().unwrap_or_default();
    match source.first_mut() {
      // save the download under the archive name, whatever the URL's file name is
      Some(first) if !first.contains("::") => *first = format!("{archive_name}::{first}"),
      Some(_) => {}
      None => source.push(archive_name),
    }
    // the first source is the generated archive; there is nothing to check
    // the other sources against
    let sha_hash = hex::encode(Sha512::digest(fs::read(package_path)?));
    let sha512sums = std::iter::once(sha_hash)
      .chain(std::iter::repeat_n("SKIP".to_string(), source.len() - 1))
      .collect();

    Ok(Self {
      // a newline would end the comment and turn the rest of the value into code
      maintainer: pacman
        .maintainer
        .clone()
        .or_else(|| settings.authors_comma_separated())
        .unwrap_or_default()
        .replace(['\r', '\n'], " "),
      pkgname,
      version: settings.version_string().to_string(),
      pkgver: pkgver(settings.version_string()),
      pkgrel,
      epoch: pacman.epoch,
      pkgdesc: settings
        .short_description()
        .trim()
        .replace(['\r', '\n'], " "),
      arch: arch.to_string(),
      url: settings.homepage_url().unwrap_or_default().to_string(),
      license: settings.license().unwrap_or_else(|| "unknown".into()),
      depends: pacman.depends.clone().unwrap_or_default(),
      optdepends: pacman.optdepends.clone().unwrap_or_default(),
      provides,
      conflicts,
      replaces: pacman.replaces.clone().unwrap_or_default(),
      source,
      sha512sums,
    })
  }

  fn pkgbuild(&self) -> String {
    let array = |values: &[String]| {
      values
        .iter()
        .map(|v| quote(v))
        .collect::<Vec<_>>()
        .join(" ")
    };
    // `source` may refer to PKGBUILD variables such as ${pkgver}, so it is double quoted
    let source = self
      .source
      .iter()
      .map(|s| double_quote(s))
      .collect::<Vec<_>>()
      .join(" ");

    let mut out = String::new();
    if !self.maintainer.is_empty() {
      out.push_str(&format!("# Maintainer: {}\n", self.maintainer));
    }
    out.push_str(&format!(
      "pkgname={}
_pkgver={}
pkgver={}
pkgrel={}
{}pkgdesc={}
arch=({})
url={}
license=({})
depends=({})
optdepends=({})
provides=({})
conflicts=({})
replaces=({})
# the binaries are already built, makepkg must not strip them or split their debug symbols
options=('!strip' '!debug')
source=({source})
sha512sums=({})

package() {{
  cp -a \"$srcdir/data/.\" \"$pkgdir/\"
}}
",
      quote(&self.pkgname),
      quote(&self.version),
      quote(&self.pkgver),
      quote(&self.pkgrel),
      self
        .epoch
        .map(|epoch| format!("epoch={epoch}\n"))
        .unwrap_or_default(),
      quote(&self.pkgdesc),
      quote(&self.arch),
      quote(&self.url),
      quote(&self.license),
      array(&self.depends),
      array(&self.optdepends),
      array(&self.provides),
      array(&self.conflicts),
      array(&self.replaces),
      array(&self.sha512sums),
    ));
    out
  }

  /// Renders the .SRCINFO, which is what `makepkg --printsrcinfo` prints for the PKGBUILD.
  fn srcinfo(&self) -> String {
    let mut out = format!("pkgbase = {}\n", self.pkgname);
    let mut field = |key: &str, value: &str| {
      if !value.is_empty() {
        out.push_str(&format!("\t{key} = {value}\n"));
      }
    };
    field("pkgdesc", &self.pkgdesc);
    field("pkgver", &self.pkgver);
    field("pkgrel", &self.pkgrel);
    if let Some(epoch) = self.epoch {
      field("epoch", &epoch.to_string());
    }
    field("url", &self.url);
    field("arch", &self.arch);
    field("license", &self.license);
    for (key, values) in [
      ("depends", &self.depends),
      ("optdepends", &self.optdepends),
      ("provides", &self.provides),
      ("conflicts", &self.conflicts),
      ("replaces", &self.replaces),
    ] {
      for value in values {
        field(key, value);
      }
    }
    field("options", "!strip");
    field("options", "!debug");
    for source in &self.source {
      field("source", &self.expand(source));
    }
    for sum in &self.sha512sums {
      field("sha512sums", sum);
    }
    out.push_str(&format!("\npkgname = {}\n", self.pkgname));
    out
  }

  /// Expands the PKGBUILD variables a `source` entry may use, as makepkg does.
  fn expand(&self, value: &str) -> String {
    let mut value = value.to_string();
    for (name, v) in [
      ("_pkgver", &self.version),
      ("pkgname", &self.pkgname),
      ("pkgver", &self.pkgver),
      ("pkgrel", &self.pkgrel),
      ("CARCH", &self.arch),
    ] {
      value = value
        .replace(&format!("${{{name}}}"), v)
        .replace(&format!("${name}"), v);
    }
    value
  }
}

/// Quotes a value as a literal bash string.
fn quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', r"'\''"))
}

/// Quotes a value as a bash string that still expands `$variables`.
fn double_quote(value: &str) -> String {
  format!(
    "\"{}\"",
    value
      .replace('\\', r"\\")
      .replace('"', "\\\"")
      .replace('`', "\\`")
  )
}

/// Whether the name is a valid `pkgname`.
///
/// See <https://wiki.archlinux.org/title/PKGBUILD#pkgname>
fn is_valid_pkgname(name: &str) -> bool {
  !name.is_empty()
    && !name.starts_with(['-', '.'])
    && name
      .chars()
      .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "@._+-".contains(c))
}

/// Whether the value is a valid `pkgrel`: a positive integer, optionally followed by a
/// period and another integer.
fn is_valid_pkgrel(pkgrel: &str) -> bool {
  let (major, minor) = pkgrel.split_once('.').unwrap_or((pkgrel, "1"));
  let is_number = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
  is_number(major) && is_number(minor) && !major.starts_with('0')
}

/// The kebab-cased product name, without the characters `pkgname` does not allow,
/// with the `-bin` suffix.
fn default_pkgname(product_name: &str) -> Option<String> {
  let name = AsKebabCase(product_name)
    .to_string()
    .chars()
    .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "@._+-".contains(*c))
    .collect::<String>();
  let name = name.trim_start_matches(['-', '.']).trim_end_matches('-');
  (!name.is_empty()).then(|| format!("{name}-bin"))
}

/// pacman does not allow hyphens in `pkgver`, so the semver pre-release
/// separator is mapped to an underscore, e.g. `1.0.0-beta.1` -> `1.0.0_beta.1`.
/// The original version is available to the PKGBUILD as `${_pkgver}`.
fn pkgver(version: &str) -> String {
  version.replace('-', "_")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn quotes_values() {
    assert_eq!(quote("gtk3>=3.24"), "'gtk3>=3.24'");
    assert_eq!(quote("it's $(id)"), r"'it'\''s $(id)'");
    assert_eq!(
      double_quote(r#"https://x/${pkgver}/"a"`b`"#),
      r#""https://x/${pkgver}/\"a\"\`b\`""#
    );
  }

  fn package() -> Package {
    Package {
      maintainer: String::new(),
      pkgname: "my-app".into(),
      version: "1.0.0-beta.1".into(),
      pkgver: "1.0.0_beta.1".into(),
      pkgrel: "2".into(),
      epoch: Some(1),
      pkgdesc: "My app".into(),
      arch: "x86_64".into(),
      url: String::new(),
      license: "MIT".into(),
      depends: vec!["gtk3".into(), "webkit2gtk-4.1>=2.40".into()],
      optdepends: vec!["libappindicator-gtk3: tray icon support".into()],
      provides: vec![],
      conflicts: vec![],
      replaces: vec![],
      source: vec!["a.tar.gz::https://x/v${_pkgver}/a_$CARCH.tar.gz".into()],
      sha512sums: vec!["abc".into()],
    }
  }

  #[test]
  fn renders_srcinfo() {
    assert_eq!(
      package().srcinfo(),
      "pkgbase = my-app
\tpkgdesc = My app
\tpkgver = 1.0.0_beta.1
\tpkgrel = 2
\tepoch = 1
\tarch = x86_64
\tlicense = MIT
\tdepends = gtk3
\tdepends = webkit2gtk-4.1>=2.40
\toptdepends = libappindicator-gtk3: tray icon support
\toptions = !strip
\toptions = !debug
\tsource = a.tar.gz::https://x/v1.0.0-beta.1/a_x86_64.tar.gz
\tsha512sums = abc

pkgname = my-app
"
    );
  }

  #[test]
  fn derives_pkgname() {
    assert_eq!(default_pkgname("My App").as_deref(), Some("my-app-bin"));
    assert_eq!(default_pkgname("Café 2").as_deref(), Some("caf-2-bin"));
    assert_eq!(default_pkgname("应用"), None);
    assert!(is_valid_pkgname("my-app-bin"));
    assert!(!is_valid_pkgname("My-App"));
    assert!(!is_valid_pkgname("-app"));
    assert!(is_valid_pkgrel("1"));
    assert!(is_valid_pkgrel("2.1"));
    assert!(!is_valid_pkgrel("0"));
    assert!(!is_valid_pkgrel("1-1"));
    assert!(!is_valid_pkgrel("1."));
  }

  #[test]
  fn maps_pkgver() {
    assert_eq!(pkgver("1.0.0"), "1.0.0");
    assert_eq!(pkgver("1.0.0-beta.1"), "1.0.0_beta.1");
  }
}
