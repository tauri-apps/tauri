// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Compiles the qtshim C++ sources against the system's Qt 6 (found with `pkg-config`).

use std::{
  env,
  path::{Path, PathBuf},
  process::Command,
};

/// Qt modules every build links.
const BASE_MODULES: &[&str] = &["Qt6Widgets", "Qt6DBus"];
/// Extra Qt modules for the `webengine` feature.
const WEBENGINE_MODULES: &[&str] = &["Qt6WebEngineWidgets", "Qt6WebChannel", "Qt6PrintSupport"];

fn main() {
  println!("cargo:rerun-if-changed=cpp");
  println!("cargo:rerun-if-env-changed=QT_MOC");
  println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

  let webengine = env::var_os("CARGO_FEATURE_WEBENGINE").is_some();
  let mut modules = BASE_MODULES.to_vec();
  if webengine {
    modules.extend_from_slice(WEBENGINE_MODULES);
  }

  let mut build = cc::Build::new();
  build
    .cpp(true)
    .std("c++17")
    .include("cpp")
    .file("cpp/core.cpp")
    .file("cpp/menu.cpp")
    .flag_if_supported("-Wno-unused-parameter");

  if webengine {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let moc_out = out_dir.join("moc_ipc_bridge.cpp");
    run_moc(Path::new("cpp/ipc_bridge.h"), &moc_out);
    build.file("cpp/webengine.cpp").file(&moc_out);
  }

  for flag in pkg_config(&[&["--cflags"], modules.as_slice()].concat()).split_whitespace() {
    if let Some(dir) = flag.strip_prefix("-I") {
      build.include(dir);
    } else if let Some(define) = flag.strip_prefix("-D") {
      match define.split_once('=') {
        Some((key, value)) => build.define(key, value),
        None => build.define(define, None),
      };
    } else {
      build.flag(flag);
    }
  }
  add_qpa_headers(&mut build);
  build.compile("qtshim");

  for flag in pkg_config(&[&["--libs"], modules.as_slice()].concat()).split_whitespace() {
    if let Some(lib) = flag.strip_prefix("-l") {
      println!("cargo:rustc-link-lib=dylib={lib}");
    } else if let Some(dir) = flag.strip_prefix("-L") {
      println!("cargo:rustc-link-search=native={dir}");
    }
  }

  // Exposed to dependents as DEP_QTSHIM_INCLUDE.
  let include = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("cpp");
  println!("cargo:include={}", include.display());
}

fn pkg_config(args: &[&str]) -> String {
  let output = Command::new("pkg-config")
    .args(args)
    .output()
    .expect("failed to run pkg-config; install it to build qtshim-sys");
  if !output.status.success() {
    panic!(
      "pkg-config {args:?} failed: {}\n\
       qtshim-sys needs the Qt 6 development files (Fedora: qt6-qtbase-devel, plus \
       qt6-qtwebengine-devel and qt6-qtwebchannel-devel for the `webengine` feature; \
       Debian/Ubuntu: qt6-base-dev, qt6-webengine-dev, qt6-webchannel-dev; Arch: qt6-base, \
       qt6-webengine, qt6-webchannel).",
      String::from_utf8_lossy(&output.stderr)
    );
  }
  String::from_utf8(output.stdout)
    .expect("pkg-config printed invalid UTF-8")
    .trim()
    .to_owned()
}

fn moc_path() -> PathBuf {
  if let Some(moc) = env::var_os("QT_MOC") {
    return moc.into();
  }
  let libexec = PathBuf::from(pkg_config(&["--variable=libexecdir", "Qt6Core"]));
  let candidate = libexec.join("moc");
  if candidate.exists() {
    return candidate;
  }
  let output = Command::new("qmake6")
    .args(["-query", "QT_INSTALL_LIBEXECS"])
    .output()
    .expect("could not find Qt's moc tool; set QT_MOC");
  PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()).join("moc")
}

fn run_moc(header: &Path, out: &Path) {
  let status = Command::new(moc_path())
    .arg(header)
    .arg("-o")
    .arg(out)
    .status()
    .expect("failed to run moc");
  assert!(status.success(), "moc failed on {}", header.display());
}

/// The QPA private header is optional; with it, Wayland `wl_surface` handles are available.
/// (Fedora: qt6-qtbase-private-devel, Debian/Ubuntu: qt6-base-private-dev.)
fn add_qpa_headers(build: &mut cc::Build) {
  let include_dir = PathBuf::from(pkg_config(&["--variable=includedir", "Qt6Gui"]));
  let version = pkg_config(&["--modversion", "Qt6Gui"]);
  let gui_private = include_dir.join("QtGui").join(&version);
  if !gui_private
    .join("QtGui/qpa/qplatformnativeinterface.h")
    .exists()
  {
    println!(
      "cargo:warning=Qt GUI private headers not found; Wayland window handles will be unavailable"
    );
    return;
  }
  let core_private = include_dir.join("QtCore").join(&version);
  build
    .include(&gui_private)
    .include(gui_private.join("QtGui"))
    .include(&core_private)
    .include(core_private.join("QtCore"))
    .define("QS_HAVE_QPA", None);
}
