#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
  if std::path::Path::new("icons/icon.ico").exists() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icons/icon.ico");
    res.compile().expect("Unable to find visual studio tools");
  } else {
    panic!("No Icon.ico found. Please add one or check the path");
  }

#[cfg(not(windows))]
fn main() {}
