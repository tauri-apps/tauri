use std::{env, path::PathBuf};

fn main() {
  let tauri_path = PathBuf::from("../../../ui");
  let mut tauri_impl_path = tauri_path.clone();

  let mut build = cc::Build::new();

  build
    .include(&tauri_path)
    .flag_if_supported("-std=c11")
    .flag_if_supported("-w");

  if env::var("DEBUG").is_err() {
    build.define("NDEBUG", None);
  } else {
    build.define("DEBUG", None);
  }

  let target = env::var("TARGET").unwrap();

  if target.contains("windows") {
    tauri_impl_path.push("tauri-windows.c");
    build.define("WEBVIEW_WINAPI", None);
    for &lib in &["ole32", "comctl32", "oleaut32", "uuid", "gdi32"] {
      println!("cargo:rustc-link-lib={}", lib);
    }
  } else if target.contains("linux") || target.contains("bsd") {
    tauri_impl_path.push("tauri-gtk.c");
    let webkit = pkg_config::Config::new()
      .atleast_version("2.8")
      .probe("webkit2gtk-4.0")
      .unwrap();

    for path in webkit.include_paths {
      build.include(path);
    }
    build.define("WEBVIEW_GTK", None);
  } else if target.contains("apple") {
    tauri_impl_path.push("tauri-cocoa.m");
    build
      .define("WEBVIEW_COCOA", None)
      .flag("-x")
      .flag("objective-c");
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=WebKit");
  } else {
    panic!("unsupported target");
  }

  build
    .file(tauri_impl_path.into_os_string().into_string().unwrap())
    .file("tauri.c");

  build.compile("tauri");
}
