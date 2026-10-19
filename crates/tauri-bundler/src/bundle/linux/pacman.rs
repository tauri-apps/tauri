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
  pkgdesc: String,
  arch: String,
  url: String,
  license: String,
  depends: Vec<String>,
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
      maintainer: settings
        .authors_comma_separated()
        .unwrap_or_default()
        .replace(['\r', '\n'], " "),
      pkgname: AsKebabCase(settings.product_name()).to_string(),
      version: settings.version_string().to_string(),
      pkgver: pkgver(settings.version_string()),
      pkgdesc: settings
        .short_description()
        .trim()
        .replace(['\r', '\n'], " "),
      arch: arch.to_string(),
      url: settings.homepage_url().unwrap_or_default().to_string(),
      license: settings.license().unwrap_or_else(|| "unknown".into()),
      depends: pacman.depends.clone().unwrap_or_default(),
      provides: pacman.provides.clone().unwrap_or_default(),
      conflicts: pacman.conflicts.clone().unwrap_or_default(),
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
pkgrel=1
pkgdesc={}
arch=({})
url={}
license=({})
depends=({})
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
      quote(&self.pkgdesc),
      quote(&self.arch),
      quote(&self.url),
      quote(&self.license),
      array(&self.depends),
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
    field("pkgrel", "1");
    field("url", &self.url);
    field("arch", &self.arch);
    field("license", &self.license);
    for (key, values) in [
      ("depends", &self.depends),
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
      ("pkgrel", &"1".to_string()),
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
      pkgdesc: "My app".into(),
      arch: "x86_64".into(),
      url: String::new(),
      license: "MIT".into(),
      depends: vec!["gtk3".into(), "webkit2gtk-4.1>=2.40".into()],
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
\tpkgrel = 1
\tarch = x86_64
\tlicense = MIT
\tdepends = gtk3
\tdepends = webkit2gtk-4.1>=2.40
\toptions = !strip
\toptions = !debug
\tsource = a.tar.gz::https://x/v1.0.0-beta.1/a_x86_64.tar.gz
\tsha512sums = abc

pkgname = my-app
"
    );
  }

  #[test]
  fn maps_pkgver() {
    assert_eq!(pkgver("1.0.0"), "1.0.0");
    assert_eq!(pkgver("1.0.0-beta.1"), "1.0.0_beta.1");
  }
}
